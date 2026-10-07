use crate::db;
use crate::models::{ResponseRule, WebhookRequest};
use axum::{
    body::Bytes,
    extract::{ConnectInfo, OriginalUri, State},
    http::{HeaderMap, Method, Response, StatusCode},
    response::IntoResponse,
    routing::any,
    Router,
};
use rand::Rng;
use regex::Regex;
use rusqlite::Connection;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tower_http::cors::{Any, CorsLayer};

pub struct ServerState {
    pub db: Arc<Mutex<Connection>>,
    pub app_handle: AppHandle,
    pub port: u16,
}

pub async fn run_server(state: Arc<ServerState>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let port = state.port;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/wh/{slug}", any(handle_webhook))
        .route("/wh/{slug}/{*tail}", any(handle_webhook))
        .route("/e/{slug}", any(handle_webhook))
        .route("/e/{slug}/{*tail}", any(handle_webhook))
        .route("/{slug}", any(handle_webhook))
        .fallback(handle_catch_all)
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    log::info!("Webhook Lab Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

async fn handle_catch_all(
    State(_state): State<Arc<ServerState>>,
    uri: OriginalUri,
    method: Method,
) -> impl IntoResponse {
    let msg = serde_json::json!({
        "app": "Webhook Lab",
        "status": "ready",
        "message": "Send webhooks to /wh/<endpoint-slug> or /e/<endpoint-slug>",
        "received_path": uri.path(),
        "received_method": method.as_str()
    });
    (StatusCode::OK, [("Content-Type", "application/json")], msg.to_string())
}

async fn handle_webhook(
    State(state): State<Arc<ServerState>>,
    method: Method,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    body_bytes: Bytes,
) -> Response<String> {
    let start_time = Instant::now();
    let path = uri.path().to_string();

    // Parse slug from path: /wh/<slug>/... or /e/<slug>/... or /<slug>/...
    let segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    let slug = if segments.is_empty() {
        "default"
    } else if (segments[0] == "wh" || segments[0] == "e") && segments.len() > 1 {
        segments[1]
    } else {
        segments[0]
    };

    // Extract headers into map
    let mut req_headers = HashMap::new();
    let mut content_type = "text/plain".to_string();
    for (k, v) in &headers {
        if let Ok(val_str) = v.to_str() {
            req_headers.insert(k.as_str().to_lowercase(), val_str.to_string());
            if k.as_str().eq_ignore_ascii_case("content-type") {
                content_type = val_str.to_string();
            }
        }
    }

    // Client IP (check Cloudflare/proxy headers if behind tunnel)
    let ip_address = req_headers
        .get("cf-connecting-ip")
        .or_else(|| req_headers.get("x-forwarded-for"))
        .cloned()
        .unwrap_or_else(|| addr.ip().to_string());

    // Query params
    let mut query_params = HashMap::new();
    if let Some(query) = uri.query() {
        for pair in query.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let Some(k) = parts.next() {
                let v = parts.next().unwrap_or("");
                query_params.insert(k.to_string(), v.to_string());
            }
        }
    }

    // Body
    let body_str = String::from_utf8(body_bytes.to_vec())
        .unwrap_or_else(|_| format!("<binary data: {} bytes>", body_bytes.len()));

    // Look up endpoint in database
    let endpoint_opt = {
        let conn = state.db.lock().unwrap();
        let endpoints = db::get_endpoints(&conn).unwrap_or_default();
        endpoints.into_iter().find(|e| e.slug == slug)
    };

    let endpoint = match endpoint_opt {
        Some(ep) => ep,
        None => {
            // Return 404 endpoint not found
            let err_body = serde_json::json!({
                "error": "Endpoint not found",
                "slug": slug,
                "hint": "Create this endpoint in Webhook Lab UI first"
            })
            .to_string();

            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("Content-Type", "application/json")
                .body(err_body)
                .unwrap();
        }
    };

    if !endpoint.is_active {
        return Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .header("Content-Type", "application/json")
            .body(r#"{"error": "Endpoint is currently disabled"}"#.to_string())
            .unwrap();
    }

    // 1. Kiểm tra phương thức HTTP cho phép (Allowed Methods)
    let method_str = method.as_str().to_uppercase();
    let is_method_allowed = endpoint.allowed_methods.is_empty()
        || endpoint
            .allowed_methods
            .iter()
            .any(|m| m.eq_ignore_ascii_case("ANY") || m.eq_ignore_ascii_case(&method_str));

    if !is_method_allowed {
        let err_body = serde_json::json!({
            "error": "Method Not Allowed",
            "received_method": method_str,
            "allowed_methods": endpoint.allowed_methods,
            "message": format!("Endpoint chỉ cho phép các phương thức: {:?}", endpoint.allowed_methods)
        })
        .to_string();

        return Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .header("Content-Type", "application/json")
            .header("Allow", endpoint.allowed_methods.join(", "))
            .body(err_body)
            .unwrap();
    }

    // 2. Tự động chuyển tiếp (Auto-Forwarding / Relay webhook giống webhook.site)
    if let Some(forward_url) = &endpoint.auto_forward_url {
        if !forward_url.trim().is_empty() {
            let fwd_url = forward_url.clone();
            let fwd_method = method.clone();
            let fwd_headers = req_headers.clone();
            let fwd_body = body_str.clone();

            tokio::spawn(async move {
                let client = reqwest::Client::builder()
                    .timeout(Duration::from_secs(10))
                    .danger_accept_invalid_certs(true)
                    .build();

                if let Ok(c) = client {
                    let mut req_builder = match fwd_method.as_str() {
                        "GET" => c.get(&fwd_url),
                        "POST" => c.post(&fwd_url),
                        "PUT" => c.put(&fwd_url),
                        "DELETE" => c.delete(&fwd_url),
                        "PATCH" => c.patch(&fwd_url),
                        _ => c.post(&fwd_url),
                    };

                    for (k, v) in &fwd_headers {
                        if !["host", "content-length"].contains(&k.to_lowercase().as_str()) {
                            req_builder = req_builder.header(k, v);
                        }
                    }

                    if !["GET", "HEAD"].contains(&fwd_method.as_str()) {
                        req_builder = req_builder.body(fwd_body);
                    }

                    let _ = req_builder.send().await;
                }
            });
        }
    }

    // 3. Chế độ Chuyển hướng HTTP Redirect (301/302/307 giống webhook.site)
    if let Some(redirect_target) = &endpoint.redirect_url {
        if !redirect_target.trim().is_empty() {
            let red_status = endpoint.redirect_status.unwrap_or(302);
            let elapsed = start_time.elapsed().as_millis() as u64;

            let request_id = format!("req-{}", uuid::Uuid::new_v4().simple());
            let req_record = WebhookRequest {
                id: request_id,
                endpoint_id: endpoint.id.clone(),
                method: method.as_str().to_string(),
                path: path.clone(),
                query_params,
                headers: req_headers,
                body: body_str,
                content_type,
                ip_address,
                timestamp: chrono::Utc::now().to_rfc3339(),
                response_status: red_status,
                response_time_ms: elapsed,
                matched_rule_id: None,
                matched_rule_name: Some("HTTP Redirect".to_string()),
                is_simulated_error: false,
            };

            {
                let conn = state.db.lock().unwrap();
                let _ = db::insert_request(&conn, &req_record);
            }

            let _ = state.app_handle.emit("request_received", req_record);

            return Response::builder()
                .status(red_status)
                .header("Location", redirect_target)
                .header("Content-Type", "text/plain")
                .body(format!("Redirecting to {}", redirect_target))
                .unwrap();
        }
    }

    // Check rules
    let rules = {
        let conn = state.db.lock().unwrap();
        db::get_rules(&conn, &endpoint.id).unwrap_or_default()
    };

    let mut matched_rule: Option<ResponseRule> = None;
    for rule in rules.into_iter().filter(|r| r.is_enabled) {
        if matches_rule(&rule, &method, &path, &req_headers, &query_params, &body_str) {
            matched_rule = Some(rule);
            break;
        }
    }

    let (status_code, resp_headers, resp_body, delay_ms, is_simulated_error) = if let Some(rule) = &matched_rule {
        (
            rule.response_status,
            rule.response_headers.clone(),
            rule.response_body.clone(),
            rule.response_delay_ms,
            false,
        )
    } else {
        // Chaos / Error simulation check
        let mut sim_status = endpoint.default_status;
        let mut sim_body = endpoint.default_body.clone();
        let mut is_simulated_error = false;

        if endpoint.error_rate_percent > 0 {
            let mut rng = rand::rng();
            let roll: u32 = rng.random_range(1..=100);
            if roll <= endpoint.error_rate_percent {
                sim_status = endpoint.error_status;
                is_simulated_error = true;
                sim_body = if let Some(custom_err) = &endpoint.error_body {
                    if !custom_err.trim().is_empty() {
                        custom_err.clone()
                    } else {
                        default_error_body(endpoint.error_status)
                    }
                } else {
                    default_error_body(endpoint.error_status)
                };
            }
        }

        (
            sim_status,
            endpoint.default_headers.clone(),
            sim_body,
            endpoint.delay_ms,
            is_simulated_error,
        )
    };

    // Apply delay if any
    if delay_ms > 0 {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    }

    let elapsed = start_time.elapsed().as_millis() as u64;

    // Save request to database
    let request_id = format!("req-{}", uuid::Uuid::new_v4().simple());
    let req_record = WebhookRequest {
        id: request_id,
        endpoint_id: endpoint.id.clone(),
        method: method.as_str().to_string(),
        path: path.clone(),
        query_params,
        headers: req_headers,
        body: body_str,
        content_type,
        ip_address,
        timestamp: chrono::Utc::now().to_rfc3339(),
        response_status: status_code,
        response_time_ms: elapsed,
        matched_rule_id: matched_rule.as_ref().map(|r| r.id.clone()),
        matched_rule_name: matched_rule.as_ref().map(|r| r.name.clone()),
        is_simulated_error,
    };

    {
        let conn = state.db.lock().unwrap();
        let _ = db::insert_request(&conn, &req_record);
    }

    // Emit live event to Tauri frontend
    let _ = state.app_handle.emit("request_received", req_record);

    // Build HTTP Response
    let mut builder = Response::builder().status(status_code);
    for (k, v) in &resp_headers {
        builder = builder.header(k, v);
    }
    if is_simulated_error {
        builder = builder.header("X-Simulated-Error", "true");
    }
    if !resp_headers.contains_key("content-type") && !resp_headers.contains_key("Content-Type") {
        builder = builder.header("Content-Type", &endpoint.default_content_type);
    }

    builder.body(resp_body).unwrap_or_else(|_| {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("Internal Server Error".to_string())
            .unwrap()
    })
}

fn matches_rule(
    rule: &ResponseRule,
    method: &Method,
    path: &str,
    headers: &HashMap<String, String>,
    query: &HashMap<String, String>,
    body: &str,
) -> bool {
    let actual_value = match rule.condition_type.as_str() {
        "method" => Some(method.as_str().to_string()),
        "path" => Some(path.to_string()),
        "header" => headers.get(&rule.condition_field.to_lowercase()).cloned(),
        "query" => query.get(&rule.condition_field).cloned(),
        "json_body" => {
            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(body) {
                // Try JSON pointer first (e.g. /event/type) or flat key
                let ptr = if rule.condition_field.starts_with('/') {
                    rule.condition_field.clone()
                } else {
                    format!("/{}", rule.condition_field.replace('.', "/"))
                };
                json_val.pointer(&ptr).map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
            } else {
                None
            }
        }
        _ => None,
    };

    match rule.condition_operator.as_str() {
        "exists" => actual_value.is_some(),
        "equals" => actual_value.map_or(false, |val| val.eq_ignore_ascii_case(&rule.condition_value)),
        "contains" => actual_value.map_or(false, |val| {
            val.to_lowercase().contains(&rule.condition_value.to_lowercase())
        }),
        "regex" => {
            if let (Some(val), Ok(re)) = (actual_value, Regex::new(&rule.condition_value)) {
                re.is_match(&val)
            } else {
                false
            }
        }
        _ => false,
    }
}

fn default_error_body(status: u16) -> String {
    let msg = match status {
        429 => "Too Many Requests - Rate limit exceeded",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        400 => "Bad Request",
        401 => "Unauthorized",
        _ => "Simulated Webhook Failure",
    };
    serde_json::json!({
        "error": msg,
        "status": status,
        "simulated": true,
        "source": "Webhook Lab Chaos Engine"
    })
    .to_string()
}
