use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Entity ánh xạ 1-1 với bảng `songs` trong `database/schema.sql`.
/// Giữ thuần khiết đúng các cột của bảng `songs`.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Song {
    pub id: String,
    pub source_key: String,
    pub title: String,
    pub artist_display: String,
    pub duration_ms: i64,
    pub audio_path: String,
    pub cover_path: Option<String>,
    pub lyrics_path: Option<String>,
    pub genre: Option<String>,
    pub bpm: Option<f64>,
    pub musical_key: Option<String>,
    pub play_count: i64,
}
