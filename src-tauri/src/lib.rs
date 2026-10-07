mod commands;
mod db;
mod models;
mod replay;
mod server;
mod tunnel;

use commands::*;
use server::ServerState;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tunnel::TunnelManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let app_handle = app.handle().clone();

            // Resolve database path in app data dir or fallback to current dir
            let app_data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            let _ = std::fs::create_dir_all(&app_data_dir);
            let db_path = app_data_dir.join("webhook_lab.db");

            log::info!("Initializing SQLite database at {:?}", db_path);
            let conn = db::init_db(&db_path).expect("Failed to initialize SQLite database");
            let db_arc = Arc::new(Mutex::new(conn));
            let tunnel_arc = Arc::new(TunnelManager::new());
            let forwarder_arc = Arc::new(tunnel::ApiForwarderManager::new());
            let port = 4567;

            // Launch Axum HTTP server in background
            let server_state = Arc::new(ServerState {
                db: db_arc.clone(),
                app_handle: app_handle.clone(),
                port,
            });

            tauri::async_runtime::spawn(async move {
                if let Err(e) = server::run_server(server_state).await {
                    log::error!("Server error: {}", e);
                }
            });

            // Register managed app state
            app.manage(AppState {
                db: db_arc,
                tunnel: tunnel_arc,
                api_forwarder: forwarder_arc,
                port,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_endpoints,
            create_endpoint,
            update_endpoint,
            delete_endpoint,
            get_requests,
            clear_requests,
            delete_request,
            get_rules,
            save_rule,
            delete_rule,
            replay_request,
            check_cloudflared_installed,
            get_tunnel_status,
            start_tunnel,
            stop_tunnel,
            get_api_forwarders,
            save_api_forwarder,
            delete_api_forwarder,
            start_api_forwarder,
            stop_api_forwarder,
            get_server_status,
            send_test_webhook,
        ])
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}
