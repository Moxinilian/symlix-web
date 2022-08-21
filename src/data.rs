use std::collections::HashMap;

pub type MusicKey = String;
pub type PlaylistKey = String;

pub struct Stream {
    name: String,
    timestamp: u64,
    vods: Vec<Vod>,
    intro_musics: Vec<MusicKey>,
    background_playlists: Vec<PlaylistKey>,
}

pub struct Vod {
    url: String,
}

pub struct Playlist {
    musics: Vec<MusicKey>,
    url: Option<String>,
}

pub struct Music {
    name: String,
    author: String,
    url: Option<String>,
}

pub type MusicDB = HashMap<MusicKey, Music>;
pub type PlaylistDB = HashMap<PlaylistKey, Playlist>;
pub type StreamDB = Vec<Stream>;
