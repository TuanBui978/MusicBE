use std::{str::FromStr, sync::Arc, time::Duration};

use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
};

use crate::config::AppConfig;

/// `AppState` tối giản chuẩn Functional Rust:
/// Chỉ giữ tài nguyên gốc của hệ thống (`config` và `db` pool).
/// Không cần bọc Service/Repository vào class thừa thãi!
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: SqlitePool,
}

impl AppState {
    pub async fn new(config: AppConfig) -> anyhow::Result<Self> {
        // Cấu hình tối ưu SQLite cho chịu tải cao (High Concurrency Read/Write):
        // - WAL (Write-Ahead Logging): Cho phép nhiều request ĐỌC cùng lúc ngay cả khi đang GHI.
        // - busy_timeout(5s): Nếu 2 luồng cùng ghi 1 lúc, luồng sau sẽ đợi tối đa 5s thay vì báo lỗi SQLITE_BUSY ngay.
        // - synchronous(Normal): Cân bằng hoàn hảo giữa tốc độ I/O và an toàn dữ liệu trong chế độ WAL.
        let connect_options = SqliteConnectOptions::from_str(&config.database_url)?
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .foreign_keys(true)
            .busy_timeout(Duration::from_secs(5));

        let db = SqlitePoolOptions::new()
            .max_connections(config.max_db_connections)
            .acquire_timeout(Duration::from_secs(5))
            .connect_with(connect_options)
            .await?;

        Ok(Self {
            config: Arc::new(config),
            db,
        })
    }
}
