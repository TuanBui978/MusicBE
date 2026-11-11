use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};

use crate::{
    error::AppResult,
    models::{
        dtos::{
            artist_dto::ArtistDetail,
            common::{ApiListResponse, PaginationParams, SearchParams},
        },
        entities::{artist::Artist, song::Song},
    },
    services::artist_service,
    state::AppState,
};

/// `GET /api/v1/artists?page=1&page_size=20`
async fn list_artists(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<ApiListResponse<Artist>>> {
    let pager = params.to_pager();
    let artists = artist_service::list_artists(&state.db, &pager).await?;
    Ok(Json(pager.into_response(artists)))
}

/// `GET /api/v1/artists/search?q=sontung&page=1`
async fn search_artists(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> AppResult<Json<ApiListResponse<Artist>>> {
    let pager = params.pagination.to_pager();
    let artists = artist_service::search_artists(&state.db, &params.q, &pager).await?;
    Ok(Json(pager.into_response(artists)))
}

/// `GET /api/v1/artists/{id}` -> Trả về chi tiết nghệ sĩ (Bio + Top 10 bài hát + Albums)
async fn get_artist_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ArtistDetail>> {
    let detail = artist_service::get_artist_by_id(&state.db, &id).await?;
    Ok(Json(detail))
}

/// `GET /api/v1/artists/{id}/songs?page=1&page_size=20` -> Xem tất cả bài hát của ca sĩ
async fn get_artist_songs(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<ApiListResponse<Song>>> {
    let pager = params.to_pager();
    let songs = artist_service::get_artist_songs(&state.db, &id, &pager).await?;
    Ok(Json(pager.into_response(songs)))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_artists))
        .route("/search", get(search_artists))
        .route("/{id}", get(get_artist_by_id))
        .route("/{id}/songs", get(get_artist_songs))
}
