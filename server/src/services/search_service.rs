use sqlx::SqlitePool;

use crate::{
    error::AppResult,
    models::dtos::search_dto::UnifiedSearchResult,
    repositories::{album_repo, artist_repo, song_repo},
};

/// Tìm kiếm tổng hợp: Chạy song song cả 3 bảng (Songs, Artists, Albums) qua `tokio::try_join!`
pub async fn unified_search(
    pool: &SqlitePool,
    query: &str,
    limit: i64,
) -> AppResult<UnifiedSearchResult> {
    let limit = limit.clamp(1, 20);

    let (songs, artists, albums) = tokio::try_join!(
        song_repo::search_by_title(pool, query, limit, 0),
        artist_repo::search_by_name(pool, query, limit, 0),
        album_repo::search_by_title(pool, query, limit, 0)
    )?;

    Ok(UnifiedSearchResult {
        songs,
        artists,
        albums,
    })
}
