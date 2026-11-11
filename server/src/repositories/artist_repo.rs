use sqlx::{Sqlite, SqlitePool};

use crate::{error::AppResult, models::entities::artist::Artist};

pub async fn find_artists_by_song_id(
    pool: &SqlitePool,
    song_id: &str,
    limit: i64,
) -> AppResult<Vec<Artist>> {
    let result = sqlx::query_as::<Sqlite, Artist>(
        r#"--sql
        SELECT a.* FROM artists a
        JOIN song_artists sa ON sa.artist_id = a.id
        WHERE sa.song_id = ?
        ORDER BY sa.is_main DESC, a.name ASC
        LIMIT ?
        "#,
    )
    .bind(song_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(result)
}

pub async fn find_all(pool: &SqlitePool, limit: i64, offset: i64) -> AppResult<Vec<Artist>> {
    let artists = sqlx::query_as::<Sqlite, Artist>(
        r#"--sql
            SELECT * FROM artists ORDER BY name ASC LIMIT ? OFFSET ?
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(artists)
}

pub async fn find_by_id(pool: &SqlitePool, id: &str) -> AppResult<Option<Artist>> {
    let artist = sqlx::query_as::<Sqlite, Artist>(
        r#"--sql
            SELECT * FROM artists WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(artist)
}

pub async fn search_by_name(
    pool: &SqlitePool,
    query: &str,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<Artist>> {
    let artists = sqlx::query_as::<Sqlite, Artist>(
        r#"--sql
            SELECT * FROM artists 
            WHERE name LIKE '%' || ? || '%' OR (real_name IS NOT NULL AND real_name LIKE '%' || ? || '%') 
            ORDER BY name ASC 
            LIMIT ? OFFSET ?
        "#,
    )
    .bind(query)
    .bind(query)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(artists)
}

pub async fn find_artists_by_album_id(
    pool: &SqlitePool,
    album_id: &str,
) -> AppResult<Vec<Artist>> {
    let result = sqlx::query_as::<Sqlite, Artist>(
        r#"--sql
        SELECT DISTINCT a.* FROM artists a
        JOIN song_artists sa ON sa.artist_id = a.id
        JOIN album_songs als ON als.song_id = sa.song_id
        WHERE als.album_id = ?
        ORDER BY a.name ASC
        "#,
    )
    .bind(album_id)
    .fetch_all(pool)
    .await?;

    Ok(result)
}
