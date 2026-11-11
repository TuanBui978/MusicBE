use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::Deserialize;

use crate::{
    error::AppResult,
    models::{
        dtos::{
            album_dto::AlbumDetail,
            common::{ApiListResponse, PaginationParams, SearchParams},
        },
        entities::{
            album::{Album, AlbumType},
            song::Song,
        },
    },
    services::album_service,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct AlbumListParams {
    pub r#type: Option<AlbumType>,
    #[serde(flatten)]
    pub pagination: PaginationParams,
}

/// `GET /api/v1/albums?type=CHART&page=1&page_size=20`
async fn list_albums(
    State(state): State<AppState>,
    Query(params): Query<AlbumListParams>,
) -> AppResult<Json<ApiListResponse<Album>>> {
    let pager = params.pagination.to_pager();
    let albums = album_service::list_albums(&state.db, params.r#type.as_ref(), &pager).await?;
    Ok(Json(pager.into_response(albums)))
}

/// `GET /api/v1/albums/search?q=top&page=1`
async fn search_albums(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> AppResult<Json<ApiListResponse<Album>>> {
    let pager = params.pagination.to_pager();
    let albums = album_service::search_albums(&state.db, &params.q, &pager).await?;
    Ok(Json(pager.into_response(albums)))
}

/// `GET /api/v1/albums/{id}` -> Trả về chi tiết Album (Album + Danh sách bài hát + Nghệ sĩ)
async fn get_album_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<AlbumDetail>> {
    let detail = album_service::get_album_by_id(&state.db, &id).await?;
    Ok(Json(detail))
}

/// `GET /api/v1/albums/{id}/songs` -> Toàn bộ bài hát trong album
async fn get_album_songs(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Vec<Song>>> {
    let songs = album_service::get_album_songs(&state.db, &id).await?;
    Ok(Json(songs))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_albums))
        .route("/search", get(search_albums))
        .route("/{id}", get(get_album_by_id))
        .route("/{id}/songs", get(get_album_songs))
}
