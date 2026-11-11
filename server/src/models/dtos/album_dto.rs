use serde::Serialize;

use crate::models::entities::{album::Album, artist::Artist, song::Song};

/// DTO trả về cho màn hình chi tiết Album / Playlist (Album Detail Screen):
/// Bao gồm thông tin Album + toàn bộ danh sách bài hát + danh sách nghệ sĩ thực hiện.
#[derive(Debug, Clone, Serialize)]
pub struct AlbumDetail {
    #[serde(flatten)]
    pub album: Album,
    pub songs: Vec<Song>,
    pub artists: Vec<Artist>,
}
