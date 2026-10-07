use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn default_200() -> u16 {
    200
}

fn default_500() -> u16 {
    500
}

fn default_body_str() -> String {
    r#"{"status": "ok", "message": "Webhook processed successfully"}"#.to_string()
}

fn default_content_type_str() -> String {
    "application/json".to_string()
}

fn default_true() -> bool {
    true
}

fn default_methods() -> Vec<String> {
    vec!["ANY".to_string()]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub id: String,
    pub name: String,
    pub slug: String,
    #[serde(default = "default_methods")]
    pub allowed_methods: Vec<String>,
    #[serde(default = "default_200")]
    pub default_status: u16,
    #[serde(default)]
    pub default_headers: HashMap<String, String>,
    #[serde(default = "default_body_str")]
    pub default_body: String,
    #[serde(default = "default_content_type_str")]
    pub default_content_type: String,
    #[serde(default)]
    pub delay_ms: u64,
    #[serde(default)]
    pub error_rate_percent: u32,
    #[serde(default = "default_500")]
    pub error_status: u16,
    #[serde(default)]
    pub error_body: Option<String>,
    #[serde(default)]
    pub redirect_url: Option<String>,
    #[serde(default)]
    pub redirect_status: Option<u16>,
    #[serde(default)]
    pub auto_forward_url: Option<String>,
    #[serde(default)]
    pub forward_response: Option<bool>,
    #[serde(default = "default_true")]
    pub is_active: bool,
    #[serde(default)]
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_count: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointInput {
    pub id: Option<String>,
    pub name: String,
    pub slug: String,
    #[serde(default = "default_methods")]
    pub allowed_methods: Vec<String>,
    #[serde(default = "default_200")]
    pub default_status: u16,
    #[serde(default)]
    pub default_headers: HashMap<String, String>,
    #[serde(default = "default_body_str")]
    pub default_body: String,
    #[serde(default = "default_content_type_str")]
    pub default_content_type: String,
    #[serde(default)]
    pub delay_ms: u64,
    #[serde(default)]
    pub error_rate_percent: u32,
    #[serde(default = "default_500")]
    pub error_status: u16,
    #[serde(default)]
    pub error_body: Option<String>,
    #[serde(default)]
    pub redirect_url: Option<String>,
    #[serde(default)]
    pub redirect_status: Option<u16>,
    #[serde(default)]
    pub auto_forward_url: Option<String>,
    #[serde(default)]
    pub forward_response: Option<bool>,
    #[serde(default)]
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookRequest {
    pub id: String,
    pub endpoint_id: String,
    pub method: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub content_type: String,
    pub ip_address: String,
    pub timestamp: String,
    pub response_status: u16,
    pub response_time_ms: u64,
    pub matched_rule_id: Option<String>,
    pub matched_rule_name: Option<String>,
    #[serde(default)]
    pub is_simulated_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseRule {
    pub id: String,
    pub endpoint_id: String,
    pub name: String,
    pub priority: i32,
    pub condition_type: String, // "header", "path", "query", "json_body", "method"
    pub condition_field: String,
    pub condition_operator: String, // "equals", "contains", "regex", "exists"
    pub condition_value: String,
    pub response_status: u16,
    pub response_headers: HashMap<String, String>,
    pub response_body: String,
    pub response_delay_ms: u64,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelStatus {
    pub is_running: bool,
    pub public_url: Option<String>,
    pub error: Option<String>,
    pub started_at: Option<String>,
    #[serde(default)]
    pub is_installing: bool,
    #[serde(default)]
    pub status_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiForwarder {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub is_running: bool,
    #[serde(default)]
    pub public_url: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub started_at: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatus {
    pub is_running: bool,
    pub port: u16,
    pub local_base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayRequestPayload {
    pub request_id: String,
    pub target_url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub time_ms: u64,
    pub error: Option<String>,
}
