pub mod album_handler;
pub mod artist_handler;
pub mod health;
pub mod search_handler;
pub mod song_handler;

use std::time::Duration;

use axum::{Router, http::StatusCode, routing::get};
use tower::limit::ConcurrencyLimitLayer;
use tower_http::{cors::CorsLayer, services::ServeDir, timeout::TimeoutLayer, trace::TraceLayer};

use crate::state::AppState;

/// Khởi tạo toàn bộ bảng định tuyến (Router) và các lớp bảo vệ (Middleware) của Server.
pub fn create_router(state: AppState) -> Router {
    let storage_dir = state.config.storage_dir.clone();
    let max_concurrent = state.config.max_concurrent_requests;
    let timeout_secs = state.config.request_timeout_secs;

    // 1. Nhóm API nghiệp vụ `/api/v1/...`
    // Modular Sub-routers: Mỗi module tự quản lý route của chính nó!
    let api_v1 = Router::new()
        .nest("/songs", song_handler::router())
        .nest("/artists", artist_handler::router())
        .nest("/albums", album_handler::router())
        .route("/search", get(search_handler::unified_search));

    // 2. Ghép toàn bộ hệ thống + Middleware chịu tải:
    // - `/storage/...`: Dùng `ServeDir` của `tower-http`, tự động hỗ trợ HTTP Range (`206 Partial Content`)
    //   để ExoPlayer/Media3 trên Android tua nhạc `.mp3` cực mượt mà không tốn RAM.
    // - `ConcurrencyLimitLayer`: Giới hạn số lượng request đồng thời trên 1 instance (Backpressure).
    // - `TimeoutLayer`: Tự động cắt các request bị treo quá thời gian quy định.
    Router::new()
        .route("/health", get(health::health_check))
        .nest("/api/v1", api_v1)
        .nest_service("/storage", ServeDir::new(storage_dir))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(timeout_secs),
        ))
        .layer(ConcurrencyLimitLayer::new(max_concurrent))
        .with_state(state)
}
