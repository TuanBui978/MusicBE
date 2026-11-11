use axum::{
    Json, Router,
    extract::{Path, Query, Request, State},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use tower::ServiceExt;
use tower_http::services::fs::ServeFile;

use crate::{
    error::{AppError, AppResult},
    models::{
        dtos::{
            common::{ApiListResponse, PaginationParams, SearchParams},
            song_dto::SongDetail,
        },
        entities::song::Song,
    },
    repositories::song_repo,
    services::song_service::{self},
    state::AppState,
};

/// Handler mẫu: `GET /api/v1/songs?page=1&page_size=20`
/// Gọi thẳng hàm trần `song_service::list_songs(&state.db, ...)` cực kỳ ngắn gọn!
async fn list_songs(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<ApiListResponse<Song>>> {
    let pager = params.to_pager();
    let songs = song_service::list_songs(&state.db, &pager).await?;

    Ok(Json(pager.into_response(songs)))
}

async fn get_song_by_id(
    State(state): State<AppState>,
    Path(params): Path<String>,
) -> AppResult<Json<SongDetail>> {
    Ok(Json(
        song_service::get_song_by_id(&state.db, &params).await?,
    ))
}

async fn search_song_by_title(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>
) -> AppResult<Json<ApiListResponse<Song>>> {
    let pager = params.pagination.to_pager();
    let songs = song_service::search_song_by_title(&state.db, &params.q, &pager).await?;
    Ok(Json(pager.into_response(songs)))
}

async fn play_song(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    song_service::play_song(&state.db, &id).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Đã tăng lượt nghe thành công"
    })))
}

/// `GET /api/v1/songs/{id}/stream`
/// Stream nhạc trực tiếp từ ID bài hát!
/// - Tự động tăng lượt nghe (`play_count`) khi bắt đầu phát.
/// - Hỗ trợ HTTP Range (`206 Partial Content`) để tua nhạc mượt mà.
/// - Ẩn hoàn toàn cấu trúc thư mục ổ cứng của server.
async fn stream_song(
    State(state): State<AppState>,
    Path(id): Path<String>,
    req: Request,
) -> AppResult<Response> {
    let song = song_repo::find_by_id(&state.db, &id).await?
        .ok_or_else(|| AppError::NotFound(format!("Không tìm thấy bài hát id = {}", id)))?;

    // Tự động tăng lượt nghe khi bắt đầu phát bài (request từ byte 0 hoặc không có header Range)
    let is_initial_play = match req.headers().get(axum::http::header::RANGE) {
        Some(val) => val.to_str().unwrap_or("").starts_with("bytes=0-"),
        None => true,
    };
    if is_initial_play {
        let _ = song_repo::increment_play_count(&state.db, &id).await;
    }

    let rel_path = song.audio_path.strip_prefix("storage/").unwrap_or(&song.audio_path);
    let full_path = std::path::Path::new(&state.config.storage_dir).join(rel_path);

    if !full_path.exists() {
        return Err(AppError::NotFound("File âm thanh không tồn tại trên hệ thống".into()));
    }

    let service = ServeFile::new(full_path);
    let res = service
        .oneshot(req)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(res.into_response())
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_songs))
        .route("/search", get(search_song_by_title))
        .route("/{id}", get(get_song_by_id))
        .route("/{id}/stream", get(stream_song))
        .route("/{id}/play", post(play_song))
}
