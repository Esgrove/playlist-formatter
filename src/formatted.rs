use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::playlist::Playlist;
use super::track::Track;
use super::types::{FileFormat, PlaylistType};
use super::utils;

/// Read a formatted CSV playlist file.
pub fn read_formatted_csv(path: &Path, data: &[BTreeMap<String, String>]) -> anyhow::Result<Playlist> {
    // TODO: fix data reading
    let playlist_name = path
        .file_stem()
        .map_or_else(String::new, |stem| stem.to_string_lossy().to_string());

    let playlist_date = None;
    let tracks = parse_formatted_tracks_from_data(data);
    let total_duration = utils::get_total_playtime(&tracks);
    let max_artist_length: usize = tracks.iter().map(Track::artist_length).max().unwrap_or(0);
    let max_title_length: usize = tracks.iter().map(Track::title_length).max().unwrap_or(0);
    let max_playtime_length: usize = utils::get_max_playtime_length(&tracks);

    Ok(Playlist {
        date: playlist_date,
        file: PathBuf::from(path),
        file_format: FileFormat::Csv,
        name: playlist_name,
        playlist_type: PlaylistType::Formatted,
        tracks,
        max_artist_length,
        max_title_length,
        max_playtime_length,
        total_duration,
    })
}

/// Parse track data from dictionary
#[must_use]
pub fn parse_formatted_tracks_from_data(data: &[BTreeMap<String, String>]) -> Vec<Track> {
    let mut tracks: Vec<Track> = Vec::new();
    for row in data {
        let artist = row.get("Artist").unwrap_or(&String::new()).clone();
        let name = row.get("Title").unwrap_or(&String::new()).clone();
        // TODO: parse times
        // let playtime = row.get("playtime");
        // let start_time = row.get("start time");
        // let end_time = row.get("end time");
        if !artist.is_empty() && !name.is_empty() {
            tracks.push(Track::new(artist, name));
        }
    }
    tracks
}
