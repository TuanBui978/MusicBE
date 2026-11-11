use serde::Serialize;

use crate::models::entities::{album::Album, artist::Artist, song::Song};

/// DTO trả về cho màn hình thông tin Nghệ sĩ (Artist Profile Screen):
/// Bao gồm tiểu sử nghệ sĩ + các bài hát nổi bật + các album phát hành.
#[derive(Debug, Clone, Serialize)]
pub struct ArtistDetail {
    #[serde(flatten)]
    pub artist: Artist,
    pub popular_songs: Vec<Song>,
    pub albums: Vec<Album>,
}
