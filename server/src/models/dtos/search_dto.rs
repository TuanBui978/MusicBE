use serde::{Deserialize, Serialize};

use crate::models::entities::{album::Album, artist::Artist, song::Song};

/// Tham số tìm kiếm tổng hợp: `GET /api/v1/search?q=...&limit=10`
#[derive(Debug, Deserialize)]
pub struct UnifiedSearchParams {
    pub q: String,
    pub limit: Option<i64>,
}

/// Kết quả tìm kiếm đồng thời cả 3 mục (Songs, Artists, Albums)
#[derive(Debug, Clone, Serialize)]
pub struct UnifiedSearchResult {
    pub songs: Vec<Song>,
    pub artists: Vec<Artist>,
    pub albums: Vec<Album>,
}
