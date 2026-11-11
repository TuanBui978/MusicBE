use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

/// Kết quả trả về chuẩn cho mọi hàm trong Service & Handler.
/// Tương đương với `Result<T>` bên Kotlin.
pub type AppResult<T> = Result<T, AppError>;

/// Quản lý lỗi tập trung cho toàn bộ Backend.
/// Dùng `thiserror` để tự động convert từ lỗi của `sqlx` hoặc `std::io` sang `AppError` bằng dấu `?`.
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum AppError {
    #[error("Không tìm thấy dữ liệu: {0}")]
    NotFound(String),

    #[error("Tham số không hợp lệ: {0}")]
    BadRequest(String),

    #[error("Server đang quá tải, vui lòng thử lại sau")]
    TooManyRequests,

    #[error("Lỗi cơ sở dữ liệu: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Lỗi đọc/ghi file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Lỗi hệ thống: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error_code: &'static str,
    message: String,
}

/// Implement `IntoResponse` để Axum tự động biến `AppError` thành HTTP Status + JSON.
/// Nhờ vậy trong Handler chỉ cần viết `service.get_song(id).await?` mà không cần `try/catch` thủ công!
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.clone()),
            AppError::TooManyRequests => (
                StatusCode::TOO_MANY_REQUESTS,
                "TOO_MANY_REQUESTS",
                self.to_string(),
            ),
            AppError::Database(err) => {
                tracing::error!("Database error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    "Lỗi truy vấn cơ sở dữ liệu".to_string(),
                )
            }
            AppError::Io(err) => {
                tracing::error!("IO error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "IO_ERROR",
                    "Lỗi truy xuất tệp tin".to_string(),
                )
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    msg.clone(),
                )
            }
        };

        let body = Json(ErrorResponse {
            success: false,
            error_code,
            message,
        });

        (status, body).into_response()
    }
}
