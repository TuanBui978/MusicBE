use axum::{
    Json,
    extract::{Query, State},
};

use crate::{
    error::AppResult,
    models::dtos::search_dto::{UnifiedSearchParams, UnifiedSearchResult},
    services::search_service,
    state::AppState,
};

/// `GET /api/v1/search?q=sontung&limit=10`
/// Tìm kiếm đồng thời cả 3 mục: Bài hát (Songs), Ca sĩ (Artists), Album/Playlist (Albums)
pub async fn unified_search(
    State(state): State<AppState>,
    Query(params): Query<UnifiedSearchParams>,
) -> AppResult<Json<UnifiedSearchResult>> {
    let limit = params.limit.unwrap_or(10);
    let result = search_service::unified_search(&state.db, &params.q, limit).await?;
    Ok(Json(result))
}
