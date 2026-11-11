use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "TEXT", rename_all = "UPPERCASE")]
pub enum AlbumType {
    Album,
    Playlist,
    Chart,
}

/// Entity ánh xạ 1-1 với bảng `albums` trong `database/schema.sql`.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Album {
    pub id: String,
    pub source_key: String,
    pub title: String,
    pub r#type: AlbumType,
    pub description: Option<String>,
    pub cover_path: Option<String>,
    pub release_date: Option<String>,
}
