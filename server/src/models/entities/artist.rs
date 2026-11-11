use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Entity ánh xạ 1-1 với bảng `artists` trong `database/schema.sql`.
#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
pub struct Artist {
    pub id: String,
    pub source_key: String,
    pub name: String,
    pub real_name: Option<String>,
    pub description: Option<String>,
    pub avatar_path: Option<String>,
    pub cover_path: Option<String>,
    pub country: Option<String>,
    pub birthday: Option<String>,
}
