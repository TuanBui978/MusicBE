use sqlx::{Sqlite, SqlitePool};

use crate::{error::AppResult, models::entities::song::Song};

/// Lấy danh sách bài hát có phân trang (`LIMIT` / `OFFSET`).
/// Không cần `JOIN` vì bảng `songs` đã có sẵn `artist_display` cho màn hình danh sách!
pub async fn find_all(pool: &SqlitePool, limit: i64, offset: i64) -> AppResult<Vec<Song>> {
    let songs = sqlx::query_as::<Sqlite, Song>(
        r#"
        SELECT * FROM songs
        ORDER BY play_count DESC, title ASC
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(songs)
}

pub async fn find_by_id(pool: &SqlitePool, id: &str) -> AppResult<Option<Song>> {
    let song = sqlx::query_as::<Sqlite, Song>("SELECT * FROM songs WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(song)
}

pub async fn search_by_title(
    pool: &SqlitePool,
    query: &str,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<Song>> {
    let result = sqlx::query_as::<Sqlite, Song>(
        r#"SELECT * FROM songs WHERE title LIKE '%' || ? || '%' ORDER BY play_count DESC, title ASC LIMIT ? OFFSET ?"#,
    )
    .bind(query)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(result)
}

pub async fn increment_play_count(pool: &SqlitePool, id: &str) -> AppResult<bool> {
    let result = sqlx::query("UPDATE songs SET play_count = play_count + 1 WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn find_songs_by_album_id(
    pool: &SqlitePool,
    album_id: &str,
) -> AppResult<Vec<Song>> {
    let result = sqlx::query_as::<Sqlite, Song>(
        r#"--sql
        SELECT s.* FROM songs s 
        JOIN album_songs als ON s.id = als.song_id 
        WHERE als.album_id = ? 
        ORDER BY als.position ASC
        "#,
    )
    .bind(album_id)
    .fetch_all(pool)
    .await?;

    Ok(result)
}

pub async fn find_songs_by_artist_id(
    pool: &SqlitePool,
    artist_id: &str,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<Song>> {
    let result = sqlx::query_as::<Sqlite, Song>(
        r#"--sql
        SELECT s.* FROM songs s
        JOIN song_artists sa ON sa.song_id = s.id
        WHERE sa.artist_id = ?
        ORDER BY s.play_count DESC, s.title ASC
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(artist_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(result)
}
