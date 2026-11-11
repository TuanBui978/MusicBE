use sqlx::SqlitePool;

use crate::{
    error::{AppError, AppResult},
    models::{
        dtos::{album_dto::AlbumDetail, common::Pager},
        entities::{
            album::{Album, AlbumType},
            song::Song,
        },
    },
    repositories::{album_repo, artist_repo, song_repo},
};

pub async fn list_albums(
    pool: &SqlitePool,
    album_type: Option<&AlbumType>,
    pager: &Pager,
) -> AppResult<Vec<Album>> {
    pager.is_valid_page()?;
    album_repo::find_all(pool, album_type, pager.limit, pager.offset).await
}

pub async fn get_album_by_id(pool: &SqlitePool, id: &str) -> AppResult<AlbumDetail> {
    // Chạy đồng thời 3 query song song (Parallel Non-blocking) bằng tokio::try_join!
    let (album, songs, artists) = tokio::try_join!(
        album_repo::find_by_id(pool, id),
        song_repo::find_songs_by_album_id(pool, id),
        artist_repo::find_artists_by_album_id(pool, id)
    )?;

    let album = album.ok_or_else(|| {
        AppError::NotFound(format!("Không tìm thấy album/playlist id = {}", id))
    })?;

    Ok(AlbumDetail {
        album,
        songs,
        artists,
    })
}

pub async fn get_album_songs(pool: &SqlitePool, album_id: &str) -> AppResult<Vec<Song>> {
    // Kiểm tra album có tồn tại không trước khi lấy bài hát
    let album = album_repo::find_by_id(pool, album_id).await?;
    if album.is_none() {
        return Err(AppError::NotFound(format!("Không tìm thấy album id = {}", album_id)));
    }
    song_repo::find_songs_by_album_id(pool, album_id).await
}

pub async fn search_albums(
    pool: &SqlitePool,
    query: &str,
    pager: &Pager,
) -> AppResult<Vec<Album>> {
    pager.is_valid_page()?;
    album_repo::search_by_title(pool, query, pager.limit, pager.offset).await
}
