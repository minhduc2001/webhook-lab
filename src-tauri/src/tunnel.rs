use crate::models::TunnelStatus;
use regex::Regex;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

pub struct TunnelManager {
    child: Arc<Mutex<Option<Child>>>,
    status: Arc<Mutex<TunnelStatus>>,
}

impl TunnelManager {
    pub fn new() -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
            status: Arc::new(Mutex::new(TunnelStatus {
                is_running: false,
                public_url: None,
                error: None,
                started_at: None,
                is_installing: false,
                status_message: None,
            })),
        }
    }

    pub async fn get_status(&self) -> TunnelStatus {
        let status = self.status.lock().await;
        status.clone()
    }

    pub async fn start(&self, port: u16, app_handle: AppHandle) -> Result<TunnelStatus, String> {
        let mut child_guard = self.child.lock().await;

        // If already running, return status
        if child_guard.is_some() {
            let status = self.status.lock().await;
            return Ok(status.clone());
        }

        // 1. Tự động kiểm tra hoặc tải cloudflared nếu máy người dùng chưa có
        let cloudflared_path = match get_or_download_cloudflared(&app_handle, &self.status).await {
            Ok(path) => path,
            Err(err) => {
                let mut status = self.status.lock().await;
                status.is_running = false;
                status.is_installing = false;
                status.error = Some(err.clone());
                status.status_message = None;
                let _ = app_handle.emit("tunnel_status_changed", status.clone());
                return Err(err);
            }
        };

        let target_url = format!("http://127.0.0.1:{}", port);

        // 2. Khởi chạy tiến trình cloudflared tunnel
        let mut cmd = Command::new(&cloudflared_path);
        cmd.args(["tunnel", "--url", &target_url]);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        #[cfg(target_os = "windows")]
        {
            // Cờ CREATE_NO_WINDOW (0x08000000) để không hiện cửa sổ command prompt đen
            cmd.creation_flags(0x08000000);
        }

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let err_msg = format!(
                    "Không thể khởi chạy cloudflared ({:?}): {}",
                    cloudflared_path, e
                );
                let mut status = self.status.lock().await;
                status.is_running = false;
                status.is_installing = false;
                status.error = Some(err_msg.clone());
                status.status_message = None;
                let _ = app_handle.emit("tunnel_status_changed", status.clone());
                return Err(err_msg);
            }
        };

        let stderr = child.stderr.take().ok_or("Không thể đọc luồng stderr của tunnel")?;
        *child_guard = Some(child);

        let status_arc = self.status.clone();
        let app_handle_clone = app_handle.clone();
        let child_arc = self.child.clone();

        // 3. Giám sát luồng stderr ở background để bắt URL https://xxxx.trycloudflare.com
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            let re = Regex::new(r"https://[a-zA-Z0-9-]+\.trycloudflare\.com").unwrap();

            while let Ok(Some(line)) = lines.next_line().await {
                log::info!("[cloudflared] {}", line);

                if let Some(matched) = re.find(&line) {
                    let url = matched.as_str().to_string();
                    let mut status = status_arc.lock().await;
                    status.is_running = true;
                    status.public_url = Some(url);
                    status.error = None;
                    status.is_installing = false;
                    status.status_message = None;
                    status.started_at = Some(chrono::Utc::now().to_rfc3339());

                    let _ = app_handle_clone.emit("tunnel_status_changed", status.clone());
                    break;
                }
            }

            // Tiếp tục theo dõi cho đến khi tiến trình dừng
            while let Ok(Some(line)) = lines.next_line().await {
                log::debug!("[cloudflared] {}", line);
            }

            // Khi tiến trình kết thúc
            let mut status = status_arc.lock().await;
            status.is_running = false;
            status.public_url = None;
            status.is_installing = false;
            status.status_message = None;
            let mut child_guard = child_arc.lock().await;
            *child_guard = None;

            let _ = app_handle_clone.emit("tunnel_status_changed", status.clone());
        });

        // Đặt trạng thái đang khởi tạo
        let mut status = self.status.lock().await;
        status.is_running = true;
        status.public_url = None;
        status.error = None;
        status.is_installing = false;
        status.status_message = Some("Đang thiết lập kết nối Cloudflare Quick Tunnel...".to_string());
        status.started_at = Some(chrono::Utc::now().to_rfc3339());
        let _ = app_handle.emit("tunnel_status_changed", status.clone());

        Ok(status.clone())
    }

    pub async fn stop(&self, app_handle: AppHandle) -> TunnelStatus {
        let mut child_guard = self.child.lock().await;
        if let Some(mut child) = child_guard.take() {
            let _ = child.kill().await;
        }

        let mut status = self.status.lock().await;
        status.is_running = false;
        status.public_url = None;
        status.error = None;
        status.started_at = None;
        status.is_installing = false;
        status.status_message = None;

        let _ = app_handle.emit("tunnel_status_changed", status.clone());
        status.clone()
    }
}

#[allow(dead_code)]
pub struct ForwarderProcess {
    pub child: Child,
    pub host: String,
    pub port: u16,
    pub public_url: Option<String>,
    pub error: Option<String>,
    pub started_at: Option<String>,
}

pub struct ApiForwarderManager {
    processes: Arc<Mutex<std::collections::HashMap<String, ForwarderProcess>>>,
}

impl ApiForwarderManager {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub async fn get_forwarder_runtime(
        &self,
        id: &str,
    ) -> Option<(bool, Option<String>, Option<String>, Option<String>)> {
        let procs = self.processes.lock().await;
        procs
            .get(id)
            .map(|p| (true, p.public_url.clone(), p.error.clone(), p.started_at.clone()))
    }

    pub async fn start(
        &self,
        id: String,
        host: String,
        port: u16,
        app_handle: AppHandle,
    ) -> Result<(), String> {
        let mut procs = self.processes.lock().await;

        if let Some(mut existing) = procs.remove(&id) {
            let _ = existing.child.kill().await;
        }

        let dummy_status = Arc::new(Mutex::new(TunnelStatus {
            is_running: false,
            public_url: None,
            error: None,
            started_at: None,
            is_installing: false,
            status_message: None,
        }));

        let cloudflared_path = match get_or_download_cloudflared(&app_handle, &dummy_status).await {
            Ok(p) => p,
            Err(e) => {
                let _ = app_handle.emit(
                    "forwarder_status_changed",
                    serde_json::json!({
                        "id": id,
                        "is_running": false,
                        "public_url": None::<String>,
                        "error": Some(e.clone()),
                    }),
                );
                return Err(e);
            }
        };

        let clean_host = if host.trim().is_empty() {
            "127.0.0.1".to_string()
        } else {
            host.trim().to_string()
        };
        let target_url = format!("http://{}:{}", clean_host, port);

        let mut cmd = Command::new(&cloudflared_path);
        cmd.args(["tunnel", "--url", &target_url]);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        #[cfg(target_os = "windows")]
        {
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let err_msg = format!("Không thể khởi chạy cloudflared: {}", e);
                let _ = app_handle.emit(
                    "forwarder_status_changed",
                    serde_json::json!({
                        "id": id,
                        "is_running": false,
                        "public_url": None::<String>,
                        "error": Some(err_msg.clone()),
                    }),
                );
                return Err(err_msg);
            }
        };

        let stderr = child.stderr.take().ok_or("Không thể đọc luồng stderr của tunnel")?;

        let started_at_str = chrono::Utc::now().to_rfc3339();
        let proc_entry = ForwarderProcess {
            child,
            host: clean_host,
            port,
            public_url: None,
            error: None,
            started_at: Some(started_at_str),
        };
        procs.insert(id.clone(), proc_entry);

        let procs_arc = self.processes.clone();
        let app_handle_clone = app_handle.clone();
        let fwd_id = id.clone();

        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            let re = Regex::new(r"https://[a-zA-Z0-9-]+\.trycloudflare\.com").unwrap();

            while let Ok(Some(line)) = lines.next_line().await {
                log::info!("[fwd-{}] {}", fwd_id, line);

                if let Some(matched) = re.find(&line) {
                    let url = matched.as_str().to_string();
                    {
                        let mut p_map = procs_arc.lock().await;
                        if let Some(p) = p_map.get_mut(&fwd_id) {
                            p.public_url = Some(url.clone());
                            p.error = None;
                        }
                    }

                    let _ = app_handle_clone.emit(
                        "forwarder_status_changed",
                        serde_json::json!({
                            "id": fwd_id.clone(),
                            "is_running": true,
                            "public_url": Some(url),
                            "error": None::<String>,
                        }),
                    );
                    break;
                }
            }

            while let Ok(Some(line)) = lines.next_line().await {
                log::debug!("[fwd-{}] {}", fwd_id, line);
            }

            {
                let mut p_map = procs_arc.lock().await;
                p_map.remove(&fwd_id);
            }

            let _ = app_handle_clone.emit(
                "forwarder_status_changed",
                serde_json::json!({
                    "id": fwd_id,
                    "is_running": false,
                    "public_url": None::<String>,
                    "error": None::<String>,
                }),
            );
        });

        let _ = app_handle.emit(
            "forwarder_status_changed",
            serde_json::json!({
                "id": id,
                "is_running": true,
                "public_url": None::<String>,
                "error": None::<String>,
            }),
        );

        Ok(())
    }

    pub async fn stop(&self, id: &str, app_handle: AppHandle) -> bool {
        let mut procs = self.processes.lock().await;
        if let Some(mut p) = procs.remove(id) {
            let _ = p.child.kill().await;
        }

        let _ = app_handle.emit(
            "forwarder_status_changed",
            serde_json::json!({
                "id": id,
                "is_running": false,
                "public_url": None::<String>,
                "error": None::<String>,
            }),
        );

        true
    }
}

/// Kiểm tra nhanh xem trên máy đã có cloudflared hay chưa (không tải gì cả)
pub async fn is_cloudflared_installed(app_handle: &AppHandle) -> bool {
    // Bước 1: Kiểm tra trong PATH
    let mut check_cmd = Command::new("cloudflared");
    check_cmd.arg("--version");
    check_cmd.stdout(Stdio::null());
    check_cmd.stderr(Stdio::null());

    #[cfg(target_os = "windows")]
    {
        check_cmd.creation_flags(0x08000000);
    }

    if let Ok(exit_status) = check_cmd.status().await {
        if exit_status.success() {
            return true;
        }
    }

    // Bước 2: Kiểm tra các thư mục chuẩn trên Windows
    #[cfg(target_os = "windows")]
    {
        let standard_paths = [
            r"C:\Program Files (x86)\cloudflared\cloudflared.exe",
            r"C:\Program Files\cloudflared\cloudflared.exe",
        ];
        for p_str in standard_paths {
            let p = PathBuf::from(p_str);
            if p.exists() && p.is_file() {
                return true;
            }
        }
    }

    // Bước 3: Kiểm tra trong thư mục bin cục bộ của app
    if let Ok(bin_dir) = get_app_bin_dir(app_handle) {
        let local_bin = if cfg!(target_os = "windows") {
            bin_dir.join("cloudflared.exe")
        } else {
            bin_dir.join("cloudflared")
        };
        if local_bin.exists() && local_bin.is_file() {
            return true;
        }
    }

    false
}

/// Lấy đường dẫn cloudflared hoặc tự động tải bản chính thức từ GitHub của Cloudflare về thư mục app data.
async fn get_or_download_cloudflared(
    app_handle: &AppHandle,
    status_arc: &Arc<Mutex<TunnelStatus>>,
) -> Result<PathBuf, String> {
    // Bước 1: Kiểm tra xem cloudflared đã có sẵn trong biến môi trường PATH chưa
    let mut check_cmd = Command::new("cloudflared");
    check_cmd.arg("--version");
    check_cmd.stdout(Stdio::null());
    check_cmd.stderr(Stdio::null());

    #[cfg(target_os = "windows")]
    {
        check_cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    if let Ok(exit_status) = check_cmd.status().await {
        if exit_status.success() {
            log::info!("Đã phát hiện cloudflared trong biến môi trường PATH.");
            return Ok(PathBuf::from("cloudflared"));
        }
    }

    // Bước 2: Kiểm tra các thư mục cài đặt tiêu chuẩn trên Windows
    #[cfg(target_os = "windows")]
    {
        let standard_paths = [
            r"C:\Program Files (x86)\cloudflared\cloudflared.exe",
            r"C:\Program Files\cloudflared\cloudflared.exe",
        ];
        for p_str in standard_paths {
            let p = PathBuf::from(p_str);
            if p.exists() && p.is_file() {
                log::info!("Đã tìm thấy cloudflared tại đường dẫn: {:?}", p);
                return Ok(p);
            }
        }
    }

    // Bước 3: Kiểm tra thư mục cục bộ của ứng dụng (app local data bin)
    let bin_dir = get_app_bin_dir(app_handle)?;
    let local_bin = if cfg!(target_os = "windows") {
        bin_dir.join("cloudflared.exe")
    } else {
        bin_dir.join("cloudflared")
    };

    if local_bin.exists() && local_bin.is_file() {
        log::info!("Đã tìm thấy cloudflared trong thư mục app: {:?}", local_bin);
        return Ok(local_bin);
    }

    // Bước 4: Chưa có ở bất cứ đâu -> Tự động tải về cho người dùng!
    log::info!("Máy chưa có cloudflared. Tiến hành tự động tải phiên bản chính thức...");

    // Cập nhật trạng thái hiển thị trên giao diện người dùng
    {
        let mut status = status_arc.lock().await;
        status.is_installing = true;
        status.status_message = Some(
            "Đang tự động tải Cloudflare Tunnel (cloudflared)... Vui lòng chờ vài giây.".to_string(),
        );
        let _ = app_handle.emit("tunnel_status_changed", status.clone());
    }

    if let Err(e) = tokio::fs::create_dir_all(&bin_dir).await {
        return Err(format!(
            "Không thể tạo thư mục lưu trữ cloudflared ({:?}): {}",
            bin_dir, e
        ));
    }

    #[cfg(target_os = "windows")]
    let download_url = "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-windows-amd64.exe";

    #[cfg(target_os = "linux")]
    let download_url = if cfg!(target_arch = "aarch64") {
        "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-arm64"
    } else {
        "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64"
    };

    #[cfg(target_os = "macos")]
    let download_url = if cfg!(target_arch = "aarch64") {
        "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-darwin-arm64.tgz"
    } else {
        "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-darwin-amd64.tgz"
    };

    let client = reqwest::Client::builder()
        .user_agent("WebhookLab/0.1.0")
        .build()
        .map_err(|e| format!("Lỗi khởi tạo HTTP client: {}", e))?;

    let res = client
        .get(download_url)
        .send()
        .await
        .map_err(|e| format!("Lỗi khi tải file từ Cloudflare GitHub: {}", e))?;

    if !res.status().is_success() {
        return Err(format!(
            "Tải cloudflared thất bại, máy chủ trả về mã HTTP {}: {}",
            res.status(),
            download_url
        ));
    }

    let bytes = res
        .bytes()
        .await
        .map_err(|e| format!("Lỗi nhận dữ liệu binary: {}", e))?;

    #[cfg(target_os = "macos")]
    {
        let tar_gz_path = bin_dir.join("cloudflared.tgz");
        tokio::fs::write(&tar_gz_path, &bytes)
            .await
            .map_err(|e| format!("Lỗi ghi file nén tgz: {}", e))?;

        let mut tar_cmd = Command::new("tar");
        tar_cmd.args(["-xzf", tar_gz_path.to_str().unwrap(), "-C", bin_dir.to_str().unwrap()]);
        let _ = tar_cmd.status().await;
        let _ = tokio::fs::remove_file(&tar_gz_path).await;
    }

    #[cfg(not(target_os = "macos"))]
    {
        let temp_bin = local_bin.with_extension("downloading");
        tokio::fs::write(&temp_bin, &bytes)
            .await
            .map_err(|e| format!("Lỗi lưu file binary: {}", e))?;

        tokio::fs::rename(&temp_bin, &local_bin)
            .await
            .map_err(|e| format!("Lỗi ghi file hoàn tất: {}", e))?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = tokio::fs::metadata(&local_bin).await {
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            let _ = tokio::fs::set_permissions(&local_bin, perms).await;
        }
    }

    log::info!("Đã tự động tải và cài đặt cloudflared thành công tại: {:?}", local_bin);

    {
        let mut status = status_arc.lock().await;
        status.is_installing = false;
        status.status_message = Some("Cài đặt Cloudflare Tunnel hoàn tất!".to_string());
        let _ = app_handle.emit("tunnel_status_changed", status.clone());
    }

    Ok(local_bin)
}

fn get_app_bin_dir(app_handle: &AppHandle) -> Result<PathBuf, String> {
    if let Ok(dir) = app_handle.path().app_local_data_dir() {
        return Ok(dir.join("bin"));
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("LOCALAPPDATA") {
            return Ok(PathBuf::from(appdata).join("WebhookLab").join("bin"));
        }
    }

    #[cfg(unix)]
    {
        if let Ok(home) = std::env::var("HOME") {
            return Ok(PathBuf::from(home).join(".local").join("share").join("webhook-lab").join("bin"));
        }
    }

    Ok(PathBuf::from(".").join("bin"))
}
