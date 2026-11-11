use sqlx::{Sqlite, SqlitePool};

use crate::{
    error::AppResult,
    models::entities::album::{Album, AlbumType},
};

/// 1. Lấy danh sách các Album chứa bài hát này (quan hệ nhiều-nhiều qua `album_songs`)
pub async fn find_albums_by_song_id(
    pool: &SqlitePool,
    song_id: &str,
    limit: i64,
) -> AppResult<Vec<Album>> {
    let result = sqlx::query_as::<Sqlite, Album>(
        r#"--sql
        SELECT a.* FROM albums a
        JOIN album_songs als ON a.id = als.album_id
        WHERE als.song_id = ?
        ORDER BY a.title ASC
        LIMIT ?
        "#,
    )
    .bind(song_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(result)
}

pub async fn find_all(
    pool: &SqlitePool,
    album_type: Option<&AlbumType>,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<Album>> {
    let mut builder = sqlx::QueryBuilder::<Sqlite>::new("SELECT * FROM albums");

    if let Some(t) = album_type {
        builder.push(" WHERE type = ").push_bind(t);
    }

    builder
        .push(" ORDER BY created_at DESC LIMIT ")
        .push_bind(limit)
        .push(" OFFSET ")
        .push_bind(offset);

    // 4. Build và fetch kết quả thẳng về Vec<Album>
    let albums = builder.build_query_as::<Album>().fetch_all(pool).await?;

    Ok(albums)
}

pub async fn find_by_id(pool: &SqlitePool, id: &str) -> AppResult<Option<Album>> {
    let album = sqlx::query_as::<Sqlite, Album>(r#"SELECT * FROM albums WHERE id = ?"#)
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(album)
}

pub async fn search_by_title(
    pool: &SqlitePool,
    query: &str,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<Album>> {
    let albums = sqlx::query_as::<Sqlite, Album>(
        r#"--sql
        SELECT * FROM albums 
        WHERE title LIKE '%' || ? || '%' 
        ORDER BY created_at DESC 
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(query)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(albums)
}

pub async fn find_albums_by_artist_id(
    pool: &SqlitePool,
    artist_id: &str,
    limit: i64,
) -> AppResult<Vec<Album>> {
    let albums = sqlx::query_as::<Sqlite, Album>(
        r#"--sql
        SELECT DISTINCT al.* FROM albums al
        JOIN album_songs als ON als.album_id = al.id
        JOIN song_artists sa ON sa.song_id = als.song_id
        WHERE sa.artist_id = ?
        ORDER BY al.created_at DESC
        LIMIT ?
        "#,
    )
    .bind(artist_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(albums)
}


