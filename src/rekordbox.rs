use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::playlist::Playlist;
use super::track::Track;
use super::types::{FileFormat, PlaylistType};
use super::utils;

/// Read data from a Rekordbox txt playlist.
pub fn read_rekordbox_txt(
    path: &Path,
    name: String,
    header: &BTreeMap<String, usize>,
    playlist_rows: &[BTreeMap<String, String>],
) -> anyhow::Result<Playlist> {
    let required_fields = ["Artist", "Track Title"];
    for field in required_fields {
        if !header.contains_key(field) {
            anyhow::bail!("Rekordbox TXT missing required field: '{field}'")
        }
    }

    let date = utils::extract_datetime_from_name(&name);

    // Rekordbox does not have any start or playtime info :(
    let mut tracks: Vec<Track> = {
        playlist_rows
            .iter()
            .filter_map(|row| {
                let artist = row.get(required_fields[0])?;
                let title = row.get(required_fields[1])?;
                if artist.is_empty() && title.is_empty() {
                    None
                } else {
                    Some(Track::new(artist.to_string(), title.to_string()))
                }
            })
            .collect()
    };

    // Remove consecutive duplicates
    tracks.dedup();

    let max_artist_length: usize = tracks.iter().map(Track::artist_length).max().unwrap_or(0);
    let max_title_length: usize = tracks.iter().map(Track::title_length).max().unwrap_or(0);

    Ok(Playlist {
        date,
        file: PathBuf::from(path),
        file_format: FileFormat::Txt,
        name,
        playlist_type: PlaylistType::Rekordbox,
        tracks,
        max_artist_length,
        max_title_length,
        max_playtime_length: 0,
        total_duration: None,
    })
}
