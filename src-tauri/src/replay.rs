use crate::models::{ReplayRequestPayload, ReplayResponse};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub async fn execute_replay(payload: ReplayRequestPayload) -> ReplayResponse {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .danger_accept_invalid_certs(true) // helpful for local dev HTTPS
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return ReplayResponse {
                status: 0,
                status_text: "Client Error".to_string(),
                headers: HashMap::new(),
                body: String::new(),
                time_ms: 0,
                error: Some(format!("Failed to build HTTP client: {}", e)),
            };
        }
    };

    let mut headers = HeaderMap::new();
    for (k, v) in &payload.headers {
        if k.eq_ignore_ascii_case("host") || k.eq_ignore_ascii_case("content-length") {
            continue;
        }
        if let (Ok(name), Ok(val)) = (
            HeaderName::from_bytes(k.as_bytes()),
            HeaderValue::from_str(v),
        ) {
            headers.insert(name, val);
        }
    }

    let method = match payload.method.to_uppercase().as_str() {
        "GET" => reqwest::Method::GET,
        "POST" => reqwest::Method::POST,
        "PUT" => reqwest::Method::PUT,
        "DELETE" => reqwest::Method::DELETE,
        "PATCH" => reqwest::Method::PATCH,
        "HEAD" => reqwest::Method::HEAD,
        "OPTIONS" => reqwest::Method::OPTIONS,
        _ => reqwest::Method::POST,
    };

    let start = Instant::now();
    let mut req_builder = client.request(method, &payload.target_url).headers(headers);

    if !["GET", "HEAD"].contains(&payload.method.to_uppercase().as_str()) {
        req_builder = req_builder.body(payload.body);
    }

    match req_builder.send().await {
        Ok(res) => {
            let status = res.status().as_u16();
            let status_text = res.status().to_string();
            let mut res_headers = HashMap::new();
            for (k, v) in res.headers() {
                if let Ok(val) = v.to_str() {
                    res_headers.insert(k.as_str().to_string(), val.to_string());
                }
            }
            let body = res.text().await.unwrap_or_default();
            let time_ms = start.elapsed().as_millis() as u64;

            ReplayResponse {
                status,
                status_text,
                headers: res_headers,
                body,
                time_ms,
                error: None,
            }
        }
        Err(e) => {
            let time_ms = start.elapsed().as_millis() as u64;
            ReplayResponse {
                status: 0,
                status_text: "Request Failed".to_string(),
                headers: HashMap::new(),
                body: String::new(),
                time_ms,
                error: Some(format!("Replay network error: {}", e)),
            }
        }
    }
}
