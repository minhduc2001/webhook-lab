use crate::db;
use crate::models::{
    ApiForwarder, Endpoint, EndpointInput, ReplayRequestPayload, ReplayResponse, ResponseRule,
    ServerStatus, TunnelStatus, WebhookRequest,
};
use crate::replay::execute_replay;
use crate::tunnel::{ApiForwarderManager, TunnelManager};
use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, State};

pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub tunnel: Arc<TunnelManager>,
    pub api_forwarder: Arc<ApiForwarderManager>,
    pub port: u16,
}

#[tauri::command]
pub async fn get_endpoints(state: State<'_, AppState>) -> Result<Vec<Endpoint>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_endpoints(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_endpoint(
    state: State<'_, AppState>,
    endpoint: EndpointInput,
) -> Result<Endpoint, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Auto-generate ID if missing
    let id = endpoint
        .id
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| format!("ep-{}", uuid::Uuid::new_v4().simple()));

    // Clean slug
    let slug = if endpoint.slug.trim().is_empty() {
        format!("hook-{}", uuid::Uuid::new_v4().simple())
    } else {
        endpoint.slug.trim().to_lowercase().replace(' ', "-")
    };

    let new_ep = Endpoint {
        id,
        name: endpoint.name,
        slug,
        allowed_methods: endpoint.allowed_methods,
        default_status: endpoint.default_status,
        default_headers: endpoint.default_headers,
        default_body: endpoint.default_body,
        default_content_type: endpoint.default_content_type,
        delay_ms: endpoint.delay_ms,
        error_rate_percent: endpoint.error_rate_percent,
        error_status: endpoint.error_status,
        error_body: endpoint.error_body,
        redirect_url: endpoint.redirect_url,
        redirect_status: endpoint.redirect_status,
        auto_forward_url: endpoint.auto_forward_url,
        forward_response: endpoint.forward_response,
        is_active: endpoint.is_active.unwrap_or(true),
        created_at: chrono::Utc::now().to_rfc3339(),
        request_count: Some(0),
    };

    db::insert_endpoint(&conn, &new_ep).map_err(|e| e.to_string())?;
    Ok(new_ep)
}

#[tauri::command]
pub async fn update_endpoint(
    state: State<'_, AppState>,
    endpoint: EndpointInput,
) -> Result<Endpoint, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let id = endpoint
        .id
        .ok_or_else(|| "Missing endpoint ID for update".to_string())?;

    let slug = if endpoint.slug.trim().is_empty() {
        format!("hook-{}", uuid::Uuid::new_v4().simple())
    } else {
        endpoint.slug.trim().to_lowercase().replace(' ', "-")
    };

    let updated_ep = Endpoint {
        id,
        name: endpoint.name,
        slug,
        allowed_methods: endpoint.allowed_methods,
        default_status: endpoint.default_status,
        default_headers: endpoint.default_headers,
        default_body: endpoint.default_body,
        default_content_type: endpoint.default_content_type,
        delay_ms: endpoint.delay_ms,
        error_rate_percent: endpoint.error_rate_percent,
        error_status: endpoint.error_status,
        error_body: endpoint.error_body,
        redirect_url: endpoint.redirect_url,
        redirect_status: endpoint.redirect_status,
        auto_forward_url: endpoint.auto_forward_url,
        forward_response: endpoint.forward_response,
        is_active: endpoint.is_active.unwrap_or(true),
        created_at: chrono::Utc::now().to_rfc3339(),
        request_count: None,
    };

    db::update_endpoint(&conn, &updated_ep).map_err(|e| e.to_string())?;
    Ok(updated_ep)
}

#[tauri::command]
pub async fn delete_endpoint(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::delete_endpoint(&conn, &id).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub async fn get_requests(
    state: State<'_, AppState>,
    endpoint_id: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<WebhookRequest>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_requests(&conn, endpoint_id.as_deref(), limit.unwrap_or(200)).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_requests(
    state: State<'_, AppState>,
    endpoint_id: Option<String>,
) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::clear_requests(&conn, endpoint_id.as_deref()).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub async fn delete_request(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::delete_request(&conn, &id).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub async fn get_rules(
    state: State<'_, AppState>,
    endpoint_id: String,
) -> Result<Vec<ResponseRule>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_rules(&conn, &endpoint_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_rule(
    state: State<'_, AppState>,
    rule: ResponseRule,
) -> Result<ResponseRule, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::insert_or_update_rule(&conn, &rule).map_err(|e| e.to_string())?;
    Ok(rule)
}

#[tauri::command]
pub async fn delete_rule(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::delete_rule(&conn, &id).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub async fn replay_request(payload: ReplayRequestPayload) -> Result<ReplayResponse, String> {
    Ok(execute_replay(payload).await)
}

#[tauri::command]
pub async fn check_cloudflared_installed(app: AppHandle) -> Result<bool, String> {
    Ok(crate::tunnel::is_cloudflared_installed(&app).await)
}

#[tauri::command]
pub async fn get_tunnel_status(state: State<'_, AppState>) -> Result<TunnelStatus, String> {
    Ok(state.tunnel.get_status().await)
}

#[tauri::command]
pub async fn start_tunnel(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<TunnelStatus, String> {
    state.tunnel.start(state.port, app).await
}

#[tauri::command]
pub async fn stop_tunnel(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<TunnelStatus, String> {
    Ok(state.tunnel.stop(app).await)
}

#[tauri::command]
pub async fn get_api_forwarders(
    state: State<'_, AppState>,
) -> Result<Vec<ApiForwarder>, String> {
    let mut forwarders = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::get_api_forwarders(&conn).map_err(|e| e.to_string())?
    };

    for fwd in &mut forwarders {
        if let Some((is_run, url, err, started)) =
            state.api_forwarder.get_forwarder_runtime(&fwd.id).await
        {
            fwd.is_running = is_run;
            fwd.public_url = url;
            fwd.error = err;
            fwd.started_at = started;
        }
    }

    Ok(forwarders)
}

#[tauri::command]
pub async fn save_api_forwarder(
    state: State<'_, AppState>,
    forwarder: ApiForwarder,
) -> Result<ApiForwarder, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut fwd = forwarder;
    if fwd.id.trim().is_empty() {
        fwd.id = format!("fwd-{}", uuid::Uuid::new_v4().simple());
    }
    if fwd.created_at.trim().is_empty() {
        fwd.created_at = chrono::Utc::now().to_rfc3339();
    }
    db::insert_or_update_forwarder(&conn, &fwd).map_err(|e| e.to_string())?;
    Ok(fwd)
}

#[tauri::command]
pub async fn delete_api_forwarder(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<bool, String> {
    state.api_forwarder.stop(&id, app).await;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::delete_forwarder(&conn, &id).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub async fn start_api_forwarder(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    host: String,
    port: u16,
) -> Result<bool, String> {
    state.api_forwarder.start(id, host, port, app).await.map(|_| true)
}

#[tauri::command]
pub async fn stop_api_forwarder(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<bool, String> {
    Ok(state.api_forwarder.stop(&id, app).await)
}

#[tauri::command]
pub async fn get_server_status(state: State<'_, AppState>) -> Result<ServerStatus, String> {
    Ok(ServerStatus {
        is_running: true,
        port: state.port,
        local_base_url: format!("http://localhost:{}", state.port),
    })
}

#[tauri::command]
pub async fn send_test_webhook(
    state: State<'_, AppState>,
    endpoint_id: String,
    preset: String,
) -> Result<WebhookRequest, String> {
    let (slug, default_status) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let endpoints = db::get_endpoints(&conn).map_err(|e| e.to_string())?;
        let ep = endpoints
            .into_iter()
            .find(|e| e.id == endpoint_id)
            .ok_or_else(|| "Endpoint not found".to_string())?;
        (ep.slug, ep.default_status)
    };

    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    headers.insert("user-agent".to_string(), "WebhookLab-Tester/1.0".to_string());

    let body = match preset.as_str() {
        "stripe" => {
            headers.insert("stripe-signature".to_string(), "t=1712750000,v1=simulated".to_string());
            serde_json::json!({
                "id": format!("evt_{}", uuid::Uuid::new_v4().simple()),
                "object": "event",
                "type": "payment_intent.succeeded",
                "created": 1712750000,
                "data": {
                    "object": {
                        "id": "pi_simulated_3399",
                        "amount": 4900,
                        "currency": "usd",
                        "status": "succeeded"
                    }
                }
            })
            .to_string()
        }
        "github" => {
            headers.insert("x-github-event".to_string(), "push".to_string());
            serde_json::json!({
                "ref": "refs/heads/main",
                "repository": {
                    "name": "webhook-lab",
                    "full_name": "octocat/webhook-lab"
                },
                "head_commit": {
                    "id": "764982a",
                    "message": "test webhook commit"
                }
            })
            .to_string()
        }
        _ => {
            serde_json::json!({
                "source": "Webhook Lab Built-in Tester",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "status": "ready"
            })
            .to_string()
        }
    };

    let test_req = WebhookRequest {
        id: format!("req-{}", uuid::Uuid::new_v4().simple()),
        endpoint_id,
        method: "POST".to_string(),
        path: format!("/wh/{}", slug),
        query_params: HashMap::new(),
        headers,
        body,
        content_type: "application/json".to_string(),
        ip_address: "127.0.0.1".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        response_status: default_status,
        response_time_ms: 5,
        matched_rule_id: None,
        matched_rule_name: None,
        is_simulated_error: false,
    };

    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::insert_request(&conn, &test_req).map_err(|e| e.to_string())?;
    }

    Ok(test_req)
}

#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
