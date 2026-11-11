use serde::Serialize;

use crate::models::entities::{album::Album, artist::Artist, song::Song};

/// DTO trả về cho màn hình chi tiết bài hát (Player Screen / Song Detail):
/// Bao gồm thông tin bài hát + danh sách nghệ sĩ tham gia + danh sách album chứa bài này.
#[derive(Debug, Clone, Serialize)]
pub struct SongDetail {
    #[serde(flatten)]
    pub song: Song,
    pub artists: Vec<Artist>,
    pub albums: Vec<Album>,
}
