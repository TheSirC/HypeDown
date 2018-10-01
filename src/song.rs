extern crate http;

use std::hash::{Hash, Hasher};
use self::http::uri::{Uri, InvalidUri};
use std::str::FromStr;

#[derive(Eq)]
pub struct Song {
    // The id given by the "API"
    id: String,
    key: String,
    artist_name: String,
    song_name: String,
    // The URL where the song can be downloaded
}

impl Song {
    fn url(&self) -> Result<Uri, InvalidUri> {
        Uri::from_str(&format!(
            "http://hypem.com/serve/source/{}/{}",
            self.id, self.key
        ))
    }

    fn filename(&self) -> String {
        let mut file_name = format!("{} - {}.mp3", self.artist_name, self.song_name);
        file_name.retain(|c| c != '/'); // Let's check for path injection right... <_<
        file_name
    }
}

impl PartialEq for Song {
    fn eq(&self, rhs: &Song) -> bool {
        self.id == rhs.id         
    }
}

impl Hash for Song {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}