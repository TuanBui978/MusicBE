mod api;
mod config;
mod error;
mod models;
mod repositories;
mod services;
mod state;

use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{config::AppConfig, state::AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Nạp biến môi trường từ file `.env`
    dotenvy::dotenv().ok();

    // 2. Khởi tạo hệ thống Log (Tracing)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "music_server=debug,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 3. Đọc cấu hình & khởi tạo AppState (Database Pool + Repositories + Services)
    let config = AppConfig::from_env();
    let bind_addr = config.bind_address();
    let state = AppState::new(config).await?;

    // 4. Tạo Router kèm các lớp Middleware (Load Shedding, Timeout, CORS, Range Streaming)
    let app = api::create_router(state.clone());

    // 5. Mở kết nối TCP và chạy Server kèm cơ chế Graceful Shutdown
    let listener = TcpListener::bind(&bind_addr).await?;
    tracing::info!(
        "🚀 MusicBE Node (PID: {}) đang lắng nghe tại http://{}",
        std::process::id(),
        bind_addr
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(state))
        .await?;

    Ok(())
}

/// Cơ chế Graceful Shutdown (Tắt máy an toàn khi cân bằng tải / redeploy):
/// Khi Load Balancer hoặc hệ điều hành gửi tín hiệu dừng (`Ctrl+C` / `SIGTERM`),
/// Server sẽ ngừng nhận kết nối mới, đợi các luồng đang stream nhạc/truy vấn DB xử lý nốt,
/// rồi đóng `SqlitePool` gọn gàng trước khi thoát hẳn.
async fn shutdown_signal(state: AppState) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Không thể đăng ký trình lắng nghe Ctrl+C");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Không thể đăng ký trình lắng nghe SIGTERM")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("🛑 Nhận tín hiệu dừng! Đang đợi các request hiện tại hoàn tất và đóng DB Pool...");
    state.db.close().await;
    tracing::info!("✅ Đã đóng kết nối an toàn.");
}
