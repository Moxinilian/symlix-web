use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub type MusicKey = String;
pub type PlaylistKey = String;

#[derive(Serialize, Deserialize, Clone)]
pub struct Stream {
    timestamp: u64,
    vods: Vec<Vod>,
    intro_music: Vec<MusicKey>,
    background_playlists: Vec<PlaylistKey>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Vod {
    url: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Playlist {
    music: Vec<MusicKey>,
    url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Music {
    title: String,
    author: String,
    url: Option<String>,
}

pub type MusicDB = HashMap<MusicKey, Music>;
pub type StreamDB = Vec<Stream>;
