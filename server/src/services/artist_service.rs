use sqlx::SqlitePool;

use crate::{
    error::{AppError, AppResult},
    models::{
        dtos::{artist_dto::ArtistDetail, common::Pager},
        entities::{artist::Artist, song::Song},
    },
    repositories::{album_repo, artist_repo, song_repo},
};

pub async fn list_artists(pool: &SqlitePool, pager: &Pager) -> AppResult<Vec<Artist>> {
    pager.is_valid_page()?;
    artist_repo::find_all(pool, pager.limit, pager.offset).await
}

pub async fn get_artist_by_id(pool: &SqlitePool, id: &str) -> AppResult<ArtistDetail> {
    // Chạy đồng thời 3 query song song (Parallel Non-blocking) bằng tokio::try_join!
    let (artist, popular_songs, albums) = tokio::try_join!(
        artist_repo::find_by_id(pool, id),
        song_repo::find_songs_by_artist_id(pool, id, 10, 0),
        album_repo::find_albums_by_artist_id(pool, id, 10)
    )?;

    let artist = artist.ok_or_else(|| {
        AppError::NotFound(format!("Không tìm thấy nghệ sĩ id = {}", id))
    })?;

    Ok(ArtistDetail {
        artist,
        popular_songs,
        albums,
    })
}

pub async fn get_artist_songs(
    pool: &SqlitePool,
    artist_id: &str,
    pager: &Pager,
) -> AppResult<Vec<Song>> {
    pager.is_valid_page()?;
    song_repo::find_songs_by_artist_id(pool, artist_id, pager.limit, pager.offset).await
}

pub async fn search_artists(
    pool: &SqlitePool,
    query: &str,
    pager: &Pager,
) -> AppResult<Vec<Artist>> {
    pager.is_valid_page()?;
    artist_repo::search_by_name(pool, query, pager.limit, pager.offset).await
}
