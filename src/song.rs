use http::uri::{InvalidUri, Uri};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize)]
pub struct Song {
// The id given by the "API"
id: String,
key: String,
artist_name: String,
song_name: String,
// The URL where the song can be downloaded
}

impl Song {
pub fn url(&self) -> Result<Uri, InvalidUri> {
    Uri::from_str(&format!(
        "http://hypem.com/serve/source/{}/{}",
        self.id, self.key
    ))
}

pub fn filename(&self) -> String {
    let mut file_name = format!("{} - {}.mp3", self.artist_name, self.song_name);
    file_name.retain(|c| c != '/'); // Let's check for path injection right... <_<
    file_name
}
}
