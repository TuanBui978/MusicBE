use axum::{Json, extract::State};
use serde::Serialize;

use crate::{error::AppResult, state::AppState};

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    /// Cổng (Port) và Process ID của node hiện tại.
    /// Khi đặt nhiều node phía sau Load Balancer (Nginx / Caddy),
    /// gọi `/health` liên tục sẽ thấy `port` và `pid` thay đổi theo thuật toán Round-Robin!
    pub instance_port: u16,
    pub process_id: u32,
    pub db_connected: bool,
}

/// Endpoint `/health` dành cho Load Balancer kiểm tra sức khỏe của từng node.
/// Nếu DB bị đứt kết nối, Load Balancer sẽ tự động ngắt lưu lượng khỏi node lỗi này.
pub async fn health_check(State(state): State<AppState>) -> AppResult<Json<HealthResponse>> {
    let db_ok = sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .is_ok();

    Ok(Json(HealthResponse {
        status: if db_ok { "healthy" } else { "degraded" },
        instance_port: state.config.port,
        process_id: std::process::id(),
        db_connected: db_ok,
    }))
}
