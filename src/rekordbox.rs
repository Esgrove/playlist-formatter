use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::playlist::Playlist;
use crate::track::Track;
use crate::types::{FileFormat, PlaylistType};
use crate::utils;

/// Read data from a Rekordbox txt playlist.
pub fn read_rekordbox_txt(
    path: &Path,
    name: String,
    header: &BTreeMap<String, usize>,
    data: &[BTreeMap<String, String>],
) -> anyhow::Result<Playlist> {
    let required_fields = ["Artist", "Track Title"];
    for field in required_fields {
        if !header.contains_key(field) {
            anyhow::bail!("Rekordbox TXT missing required field: '{}'", field)
        }
    }

    let date = utils::extract_datetime_from_name(&name);

    // Rekordbox does not have any start or play time info :(
    let mut tracks: Vec<Track> = {
        data.iter()
            .map(|row| {
                Track::new(
                    row.get(required_fields[0]).unwrap().to_string(),
                    row.get(required_fields[1]).unwrap().to_string(),
                )
            })
            .collect()
    };

    // Remove consecutive duplicates
    tracks.dedup();

    let max_artist_length: usize = tracks.iter().map(|t| t.artist_length()).max().unwrap_or(0);
    let max_title_length: usize = tracks.iter().map(|t| t.title_length()).max().unwrap_or(0);

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
