use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Track {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub file_path: String,
    pub cover_path: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTrackRequest {
    pub title: Option<String>,
    pub artist: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TrackResponse {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub cover_url: Option<String>,
    pub stream_url: String,
    pub created_at: String,
}

impl Track {
    pub fn to_response(&self, base_url: &str) -> TrackResponse {
        TrackResponse {
            id: self.id,
            title: self.title.clone(),
            artist: self.artist.clone(),
            cover_url: self
                .cover_path
                .as_ref()
                .map(|_| format!("{}/api/v1/tracks/{}/cover", base_url, self.id)),
            stream_url: format!("{}/api/v1/tracks/{}/stream", base_url, self.id),
            created_at: self.created_at.clone(),
        }
    }
}
