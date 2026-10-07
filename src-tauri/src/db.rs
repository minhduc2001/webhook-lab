use crate::models::{ApiForwarder, Endpoint, ResponseRule, WebhookRequest};
use rusqlite::{params, Connection, Result};
use std::collections::HashMap;
use std::path::Path;

pub fn init_db<P: AsRef<Path>>(path: P) -> Result<Connection> {
    let conn = Connection::open(path)?;

    // Enable WAL mode for high performance and low lock contention
    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS endpoints (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            slug TEXT UNIQUE NOT NULL,
            allowed_methods TEXT NOT NULL DEFAULT '[\"ANY\"]',
            default_status INTEGER NOT NULL,
            default_headers TEXT NOT NULL,
            default_body TEXT NOT NULL,
            default_content_type TEXT NOT NULL,
            delay_ms INTEGER NOT NULL DEFAULT 0,
            error_rate_percent INTEGER NOT NULL DEFAULT 0,
            error_status INTEGER NOT NULL DEFAULT 500,
            error_body TEXT,
            redirect_url TEXT,
            redirect_status INTEGER DEFAULT 302,
            auto_forward_url TEXT,
            forward_response INTEGER DEFAULT 0,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS requests (
            id TEXT PRIMARY KEY,
            endpoint_id TEXT NOT NULL,
            method TEXT NOT NULL,
            path TEXT NOT NULL,
            query_params TEXT NOT NULL,
            headers TEXT NOT NULL,
            body TEXT NOT NULL,
            content_type TEXT NOT NULL,
            ip_address TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            response_status INTEGER NOT NULL,
            response_time_ms INTEGER NOT NULL,
            matched_rule_id TEXT,
            matched_rule_name TEXT,
            is_simulated_error INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY(endpoint_id) REFERENCES endpoints(id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS rules (
            id TEXT PRIMARY KEY,
            endpoint_id TEXT NOT NULL,
            name TEXT NOT NULL,
            priority INTEGER NOT NULL DEFAULT 1,
            condition_type TEXT NOT NULL,
            condition_field TEXT NOT NULL,
            condition_operator TEXT NOT NULL,
            condition_value TEXT NOT NULL,
            response_status INTEGER NOT NULL DEFAULT 200,
            response_headers TEXT NOT NULL,
            response_body TEXT NOT NULL,
            response_delay_ms INTEGER NOT NULL DEFAULT 0,
            is_enabled INTEGER NOT NULL DEFAULT 1,
            FOREIGN KEY(endpoint_id) REFERENCES endpoints(id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS api_forwarders (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            host TEXT NOT NULL DEFAULT '127.0.0.1',
            port INTEGER NOT NULL,
            created_at TEXT NOT NULL
        )",
        [],
    )?;

    // Soft migrations for existing databases
    let _ = conn.execute("ALTER TABLE endpoints ADD COLUMN error_body TEXT;", []);
    let _ = conn.execute("ALTER TABLE endpoints ADD COLUMN allowed_methods TEXT DEFAULT '[\"ANY\"]';", []);
    let _ = conn.execute("ALTER TABLE endpoints ADD COLUMN redirect_url TEXT;", []);
    let _ = conn.execute("ALTER TABLE endpoints ADD COLUMN redirect_status INTEGER DEFAULT 302;", []);
    let _ = conn.execute("ALTER TABLE endpoints ADD COLUMN auto_forward_url TEXT;", []);
    let _ = conn.execute("ALTER TABLE endpoints ADD COLUMN forward_response INTEGER DEFAULT 0;", []);
    let _ = conn.execute("ALTER TABLE requests ADD COLUMN is_simulated_error INTEGER NOT NULL DEFAULT 0;", []);

    // Seed default endpoint if empty
    let count: i64 = conn.query_row("SELECT count(*) FROM endpoints", [], |row| row.get(0))?;
    if count == 0 {
        let ep = Endpoint {
            id: "ep-default".to_string(),
            name: "Default Webhook".to_string(),
            slug: "default".to_string(),
            allowed_methods: vec!["ANY".to_string()],
            default_status: 200,
            default_headers: {
                let mut h = HashMap::new();
                h.insert("Content-Type".to_string(), "application/json".to_string());
                h.insert("X-Powered-By".to_string(), "WebhookLab".to_string());
                h
            },
            default_body: r#"{"status": "ok", "message": "Webhook received successfully"}"#.to_string(),
            default_content_type: "application/json".to_string(),
            delay_ms: 0,
            error_rate_percent: 0,
            error_status: 500,
            error_body: None,
            redirect_url: None,
            redirect_status: Some(302),
            auto_forward_url: None,
            forward_response: Some(false),
            is_active: true,
            created_at: chrono::Utc::now().to_rfc3339(),
            request_count: Some(0),
        };
        insert_endpoint(&conn, &ep)?;
    }

    Ok(conn)
}

pub fn get_endpoints(conn: &Connection) -> Result<Vec<Endpoint>> {
    let mut stmt = conn.prepare(
        "SELECT e.id, e.name, e.slug, e.default_status, e.default_headers, e.default_body,
                e.default_content_type, e.delay_ms, e.error_rate_percent, e.error_status,
                e.error_body, e.allowed_methods, e.redirect_url, e.redirect_status,
                e.auto_forward_url, e.forward_response, e.is_active, e.created_at,
                (SELECT COUNT(*) FROM requests r WHERE r.endpoint_id = e.id) as req_count
         FROM endpoints e
         ORDER BY e.created_at ASC"
    )?;

    let rows = stmt.query_map([], |row| {
        let headers_json: String = row.get(4)?;
        let headers: HashMap<String, String> = serde_json::from_str(&headers_json).unwrap_or_default();
        let methods_json: Option<String> = row.get(11)?;
        let allowed_methods: Vec<String> = methods_json
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| vec!["ANY".to_string()]);

        let forward_resp_int: Option<i32> = row.get(15)?;
        let is_active_int: i32 = row.get(16)?;
        let req_count: i64 = row.get(18)?;

        Ok(Endpoint {
            id: row.get(0)?,
            name: row.get(1)?,
            slug: row.get(2)?,
            default_status: row.get::<_, u16>(3)?,
            default_headers: headers,
            default_body: row.get(5)?,
            default_content_type: row.get(6)?,
            delay_ms: row.get::<_, u64>(7)?,
            error_rate_percent: row.get::<_, u32>(8)?,
            error_status: row.get::<_, u16>(9)?,
            error_body: row.get(10)?,
            allowed_methods,
            redirect_url: row.get(12)?,
            redirect_status: row.get(13)?,
            auto_forward_url: row.get(14)?,
            forward_response: forward_resp_int.map(|i| i == 1),
            is_active: is_active_int == 1,
            created_at: row.get(17)?,
            request_count: Some(req_count as u64),
        })
    })?;

    let mut endpoints = Vec::new();
    for r in rows {
        endpoints.push(r?);
    }
    Ok(endpoints)
}

pub fn insert_endpoint(conn: &Connection, ep: &Endpoint) -> Result<()> {
    let headers_json = serde_json::to_string(&ep.default_headers).unwrap_or_else(|_| "{}".to_string());
    let methods_json = serde_json::to_string(&ep.allowed_methods).unwrap_or_else(|_| "[\"ANY\"]".to_string());

    conn.execute(
        "INSERT INTO endpoints (id, name, slug, allowed_methods, default_status, default_headers,
                                default_body, default_content_type, delay_ms, error_rate_percent,
                                error_status, error_body, redirect_url, redirect_status,
                                auto_forward_url, forward_response, is_active, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
        params![
            ep.id,
            ep.name,
            ep.slug,
            methods_json,
            ep.default_status,
            headers_json,
            ep.default_body,
            ep.default_content_type,
            ep.delay_ms,
            ep.error_rate_percent,
            ep.error_status,
            ep.error_body,
            ep.redirect_url,
            ep.redirect_status.unwrap_or(302),
            ep.auto_forward_url,
            if ep.forward_response.unwrap_or(false) { 1 } else { 0 },
            if ep.is_active { 1 } else { 0 },
            ep.created_at,
        ],
    )?;
    Ok(())
}

pub fn update_endpoint(conn: &Connection, ep: &Endpoint) -> Result<()> {
    let headers_json = serde_json::to_string(&ep.default_headers).unwrap_or_else(|_| "{}".to_string());
    let methods_json = serde_json::to_string(&ep.allowed_methods).unwrap_or_else(|_| "[\"ANY\"]".to_string());

    conn.execute(
        "UPDATE endpoints
         SET name = ?1, slug = ?2, allowed_methods = ?3, default_status = ?4, default_headers = ?5,
             default_body = ?6, default_content_type = ?7, delay_ms = ?8,
             error_rate_percent = ?9, error_status = ?10, error_body = ?11,
             redirect_url = ?12, redirect_status = ?13, auto_forward_url = ?14,
             forward_response = ?15, is_active = ?16
         WHERE id = ?17",
        params![
            ep.name,
            ep.slug,
            methods_json,
            ep.default_status,
            headers_json,
            ep.default_body,
            ep.default_content_type,
            ep.delay_ms,
            ep.error_rate_percent,
            ep.error_status,
            ep.error_body,
            ep.redirect_url,
            ep.redirect_status.unwrap_or(302),
            ep.auto_forward_url,
            if ep.forward_response.unwrap_or(false) { 1 } else { 0 },
            if ep.is_active { 1 } else { 0 },
            ep.id,
        ],
    )?;
    Ok(())
}

pub fn delete_endpoint(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM rules WHERE endpoint_id = ?1", params![id])?;
    conn.execute("DELETE FROM requests WHERE endpoint_id = ?1", params![id])?;
    conn.execute("DELETE FROM endpoints WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn insert_request(conn: &Connection, req: &WebhookRequest) -> Result<()> {
    let qp_json = serde_json::to_string(&req.query_params).unwrap_or_else(|_| "{}".to_string());
    let headers_json = serde_json::to_string(&req.headers).unwrap_or_else(|_| "{}".to_string());

    conn.execute(
        "INSERT INTO requests (id, endpoint_id, method, path, query_params, headers, body,
                               content_type, ip_address, timestamp, response_status,
                               response_time_ms, matched_rule_id, matched_rule_name, is_simulated_error)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        params![
            req.id,
            req.endpoint_id,
            req.method,
            req.path,
            qp_json,
            headers_json,
            req.body,
            req.content_type,
            req.ip_address,
            req.timestamp,
            req.response_status,
            req.response_time_ms,
            req.matched_rule_id,
            req.matched_rule_name,
            if req.is_simulated_error { 1 } else { 0 },
        ],
    )?;
    Ok(())
}

pub fn get_requests(conn: &Connection, endpoint_id: Option<&str>, limit: usize) -> Result<Vec<WebhookRequest>> {
    let sql = match endpoint_id {
        Some(_) => {
            "SELECT id, endpoint_id, method, path, query_params, headers, body,
                    content_type, ip_address, timestamp, response_status, response_time_ms,
                    matched_rule_id, matched_rule_name, is_simulated_error
             FROM requests
             WHERE endpoint_id = ?1
             ORDER BY timestamp DESC
             LIMIT ?2"
        }
        None => {
            "SELECT id, endpoint_id, method, path, query_params, headers, body,
                    content_type, ip_address, timestamp, response_status, response_time_ms,
                    matched_rule_id, matched_rule_name, is_simulated_error
             FROM requests
             ORDER BY timestamp DESC
             LIMIT ?1"
        }
    };

    let mut stmt = conn.prepare(sql)?;
    let mut requests = Vec::new();

    if let Some(ep_id) = endpoint_id {
        let rows = stmt.query_map(params![ep_id, limit as i64], map_request_row)?;
        for r in rows {
            requests.push(r?);
        }
    } else {
        let rows = stmt.query_map(params![limit as i64], map_request_row)?;
        for r in rows {
            requests.push(r?);
        }
    }

    Ok(requests)
}

fn map_request_row(row: &rusqlite::Row) -> Result<WebhookRequest> {
    let qp_str: String = row.get(4)?;
    let hdrs_str: String = row.get(5)?;

    let query_params: HashMap<String, String> = serde_json::from_str(&qp_str).unwrap_or_default();
    let headers: HashMap<String, String> = serde_json::from_str(&hdrs_str).unwrap_or_default();
    let is_simulated_error_int: i32 = row.get(14).unwrap_or(0);

    Ok(WebhookRequest {
        id: row.get(0)?,
        endpoint_id: row.get(1)?,
        method: row.get(2)?,
        path: row.get(3)?,
        query_params,
        headers,
        body: row.get(6)?,
        content_type: row.get(7)?,
        ip_address: row.get(8)?,
        timestamp: row.get(9)?,
        response_status: row.get::<_, u16>(10)?,
        response_time_ms: row.get::<_, u64>(11)?,
        matched_rule_id: row.get(12)?,
        matched_rule_name: row.get(13)?,
        is_simulated_error: is_simulated_error_int == 1,
    })
}

pub fn clear_requests(conn: &Connection, endpoint_id: Option<&str>) -> Result<()> {
    if let Some(id) = endpoint_id {
        conn.execute("DELETE FROM requests WHERE endpoint_id = ?1", params![id])?;
    } else {
        conn.execute("DELETE FROM requests", [])?;
    }
    Ok(())
}

pub fn delete_request(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM requests WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn get_rules(conn: &Connection, endpoint_id: &str) -> Result<Vec<ResponseRule>> {
    let mut stmt = conn.prepare(
        "SELECT id, endpoint_id, name, priority, condition_type, condition_field,
                condition_operator, condition_value, response_status, response_headers,
                response_body, response_delay_ms, is_enabled
         FROM rules
         WHERE endpoint_id = ?1
         ORDER BY priority ASC",
    )?;

    let rows = stmt.query_map(params![endpoint_id], |row| {
        let hdrs_str: String = row.get(9)?;
        let response_headers: HashMap<String, String> = serde_json::from_str(&hdrs_str).unwrap_or_default();
        let is_enabled_int: i32 = row.get(12)?;

        Ok(ResponseRule {
            id: row.get(0)?,
            endpoint_id: row.get(1)?,
            name: row.get(2)?,
            priority: row.get(3)?,
            condition_type: row.get(4)?,
            condition_field: row.get(5)?,
            condition_operator: row.get(6)?,
            condition_value: row.get(7)?,
            response_status: row.get::<_, u16>(8)?,
            response_headers,
            response_body: row.get(10)?,
            response_delay_ms: row.get::<_, u64>(11)?,
            is_enabled: is_enabled_int == 1,
        })
    })?;

    let mut rules = Vec::new();
    for r in rows {
        rules.push(r?);
    }
    Ok(rules)
}

pub fn insert_or_update_rule(conn: &Connection, rule: &ResponseRule) -> Result<()> {
    let hdrs_json = serde_json::to_string(&rule.response_headers).unwrap_or_else(|_| "{}".to_string());
    conn.execute(
        "INSERT INTO rules (id, endpoint_id, name, priority, condition_type, condition_field,
                            condition_operator, condition_value, response_status, response_headers,
                            response_body, response_delay_ms, is_enabled)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            priority = excluded.priority,
            condition_type = excluded.condition_type,
            condition_field = excluded.condition_field,
            condition_operator = excluded.condition_operator,
            condition_value = excluded.condition_value,
            response_status = excluded.response_status,
            response_headers = excluded.response_headers,
            response_body = excluded.response_body,
            response_delay_ms = excluded.response_delay_ms,
            is_enabled = excluded.is_enabled",
        params![
            rule.id,
            rule.endpoint_id,
            rule.name,
            rule.priority,
            rule.condition_type,
            rule.condition_field,
            rule.condition_operator,
            rule.condition_value,
            rule.response_status,
            hdrs_json,
            rule.response_body,
            rule.response_delay_ms,
            if rule.is_enabled { 1 } else { 0 },
        ],
    )?;
    Ok(())
}

pub fn delete_rule(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM rules WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn get_api_forwarders(conn: &Connection) -> Result<Vec<ApiForwarder>> {
    let mut stmt = conn.prepare("SELECT id, name, host, port, created_at FROM api_forwarders ORDER BY created_at ASC")?;
    let forwarders = stmt.query_map([], |row| {
        Ok(ApiForwarder {
            id: row.get(0)?,
            name: row.get(1)?,
            host: row.get(2)?,
            port: row.get(3)?,
            created_at: row.get(4)?,
            is_running: false,
            public_url: None,
            error: None,
            started_at: None,
        })
    })?
    .filter_map(|r| r.ok())
    .collect();
    Ok(forwarders)
}

pub fn insert_or_update_forwarder(conn: &Connection, fwd: &ApiForwarder) -> Result<()> {
    conn.execute(
        "INSERT INTO api_forwarders (id, name, host, port, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            host = excluded.host,
            port = excluded.port",
        params![fwd.id, fwd.name, fwd.host, fwd.port, fwd.created_at],
    )?;
    Ok(())
}

pub fn delete_forwarder(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM api_forwarders WHERE id = ?1", params![id])?;
    Ok(())
}
