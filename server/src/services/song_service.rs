use sqlx::SqlitePool;

use crate::{
    error::{AppError, AppResult},
    models::{
        dtos::{common::Pager, song_dto::SongDetail},
        entities::song::Song,
    },
    repositories::{album_repo, artist_repo, song_repo},
};

pub async fn list_songs(pool: &SqlitePool, pager: &Pager) -> AppResult<Vec<Song>> {
    pager.is_valid_page()?;
    song_repo::find_all(pool, pager.limit, pager.offset).await
}

pub async fn get_song_by_id(pool: &SqlitePool, id: &str) -> AppResult<SongDetail> {
    let (song, artists, albums) = tokio::try_join!(
        song_repo::find_by_id(pool, id),
        artist_repo::find_artists_by_song_id(pool, id, 5),
        album_repo::find_albums_by_song_id(pool, id, 5)
    )?;
    let song = song.ok_or_else(|| {
        AppError::NotFound(format!("Không tìm thấy bài hát id = {}", id))
    })?;

    Ok(SongDetail {
        song,
        artists,
        albums,
    })
}

pub async fn search_song_by_title(pool: &SqlitePool, query: &str, pager: &Pager) -> AppResult<Vec<Song>> {
    pager.is_valid_page()?;
    song_repo::search_by_title(pool, query, pager.limit, pager.offset).await
}

pub async fn play_song(pool: &SqlitePool, id: &str) -> AppResult<()> {
    let updated = song_repo::increment_play_count(pool, id).await?;
    if !updated {
        return Err(AppError::NotFound(format!("Không tìm thấy bài hát id = {}", id)));
    }
    Ok(())
}

// TODO 3: Lấy danh sách bài hát của 1 nghệ sĩ (cho trang profile ca sĩ)
// pub async fn get_artist_songs(pool: &SqlitePool, artist_id: &str, pager: &Pager) -> AppResult<Vec<Song>> {
//     pager.is_valid_page()?;
//     song_repo::find_songs_by_artist_id(pool, artist_id, pager.limit, pager.offset).await
// }
