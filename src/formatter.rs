use chrono::{DateTime, Duration, Utc};
use std::fs::File;
use std::fmt;

#[derive(Debug)]
enum PlaylistType {
    Rekordbox,
    Serato,
}

#[derive(Debug)]
enum PlaylistFormat {
    Txt,
    Csv,
}

#[derive(Debug)]
struct Track {
    title: String,
    artist: String,
    start_time: Optional<DateTime>,
    play_time: Optional<Duration>,
}

#[derive(Debug)]
struct Playlist {
    date: DateTime,
    file: File,
    format: PlaylistFormat,
    name: String,
    playlist_type: PlaylistType,
    tracks: Vec<Track>
}

impl fmt::Display for PlaylistType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for PlaylistFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
