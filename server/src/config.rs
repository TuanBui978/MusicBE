use std::env;
use std::path::PathBuf;

/// Cấu hình toàn cục của Server (đọc từ file `.env` hoặc biến môi trường hệ thống).
///
/// Khi chạy nhiều instance đứng sau Load Balancer (Nginx / Caddy),
/// ta chỉ cần đổi biến `PORT=8081`, `PORT=8082`... là chạy được nhiều node song song.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub storage_dir: PathBuf,
    /// Số kết nối tối đa trong DB Pool (SQLite chế độ WAL đọc song song rất tốt)
    pub max_db_connections: u32,
    /// Giới hạn số request xử lý đồng thời trên 1 node (chống sập server khi quá tải - Backpressure)
    pub max_concurrent_requests: usize,
    /// Thời gian tối đa cho 1 request thông thường (giây) trước khi tự ngắt
    pub request_timeout_secs: u64,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            database_url: env::var("DATABASE_URL")
                .expect("Thiếu biến môi trường DATABASE_URL trong .env"),
            storage_dir: PathBuf::from(
                env::var("STORAGE_DIR").unwrap_or_else(|_| "../storage".to_string()),
            ),
            max_db_connections: env::var("MAX_DB_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(16),
            max_concurrent_requests: env::var("MAX_CONCURRENT_REQUESTS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(500),
            request_timeout_secs: env::var("REQUEST_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
        }
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
