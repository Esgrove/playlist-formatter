use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use chrono::{NaiveDateTime, NaiveTime, TimeDelta, Timelike};

use super::playlist::Playlist;
use super::track::Track;
use super::types::{FileFormat, PlaylistType};
use super::{serato, utils};

/// Read a Serato CSV playlist file.
pub fn read_serato_csv(path: &Path, data: Vec<BTreeMap<String, String>>) -> anyhow::Result<Playlist> {
    let (playlist_name, playlist_date) = serato::parse_serato_playlist_info(&data[0]);
    let tracks = serato::parse_serato_tracks_from_data(&data, playlist_date);
    let total_duration = utils::get_total_playtime(&tracks);
    let max_artist_length: usize = tracks.iter().map(|t| t.artist_length()).max().unwrap_or(0);
    let max_title_length: usize = tracks.iter().map(|t| t.title_length()).max().unwrap_or(0);
    let max_playtime_length: usize = utils::get_max_playtime_length(&tracks);

    Ok(Playlist {
        date: playlist_date,
        file: PathBuf::from(path),
        file_format: FileFormat::Csv,
        name: playlist_name,
        playlist_type: PlaylistType::Serato,
        tracks,
        max_artist_length,
        max_title_length,
        max_playtime_length,
        total_duration,
    })
}

/// Read data from a Serato txt playlist.
pub fn read_serato_txt(
    path: &Path,
    name: String,
    header: &BTreeMap<String, usize>,
    data: &[BTreeMap<String, String>],
) -> anyhow::Result<Playlist> {
    let required_fields = ["artist", "name"];
    for field in required_fields {
        if !header.contains_key(field) {
            anyhow::bail!("Serato TXT missing required field: '{}'", field)
        }
    }

    let (playlist_name, playlist_date) = serato::parse_serato_playlist_info(&data[0]);
    let name = if playlist_name.is_empty() { name } else { playlist_name };
    let date = if playlist_date.is_none() {
        utils::extract_datetime_from_name(&name)
    } else {
        playlist_date
    };
    let tracks = serato::parse_serato_tracks_from_data(data, playlist_date);
    let total_duration = utils::get_total_playtime(&tracks);
    let max_artist_length: usize = tracks.iter().map(|t| t.artist_length()).max().unwrap_or(0);
    let max_title_length: usize = tracks.iter().map(|t| t.title_length()).max().unwrap_or(0);
    let max_playtime_length: usize = utils::get_max_playtime_length(&tracks);

    Ok(Playlist {
        date,
        file: PathBuf::from(path),
        file_format: FileFormat::Txt,
        name,
        playlist_type: PlaylistType::Serato,
        tracks,
        max_artist_length,
        max_title_length,
        max_playtime_length,
        total_duration,
    })
}

/// Parse first row data from a Serato playlist.
///
/// This row should contain the playlist name and start datetime.
pub fn parse_serato_playlist_info(data: &BTreeMap<String, String>) -> (String, Option<NaiveDateTime>) {
    let playlist_name = match data.get("name") {
        None => String::new(),
        Some(n) => n.to_string(),
    };
    // timestamp, for example "10.01.2019, 20.00.00 EET"
    let mut playlist_date = match data.get("start time") {
        None => None,
        Some(time) => NaiveDateTime::parse_from_str(time, "%d.%m.%Y, %H.%M.%S %Z").ok(),
    };
    if playlist_date.is_none() && !playlist_name.is_empty() {
        playlist_date = utils::extract_datetime_from_name(&playlist_name);
    }
    (playlist_name, playlist_date)
}

/// Parse Serato track data from dictionary
pub fn parse_serato_tracks_from_data(
    data: &[BTreeMap<String, String>],
    playlist_date: Option<NaiveDateTime>,
) -> Vec<Track> {
    let start_date = playlist_date.unwrap_or_default().date();
    let initial_tracks: Vec<Track> = {
        data[1..]
            .iter()
            .map(|row| {
                let start_time: Option<NaiveDateTime> = row
                    .get("start time")
                    .and_then(|t| NaiveTime::parse_from_str(t, "%H.%M.%S %Z").ok())
                    .map(|n| NaiveDateTime::new(start_date, n));

                let end_time: Option<NaiveDateTime> = row
                    .get("end time")
                    .and_then(|t| NaiveTime::parse_from_str(t, "%H.%M.%S %Z").ok())
                    .map(|n| NaiveDateTime::new(start_date, n));

                let play_time = match row.get("playtime") {
                    Some(t) => NaiveTime::parse_from_str(t, "%H:%M:%S").ok().and_then(|n| {
                        let hours = TimeDelta::try_hours(i64::from(n.hour()))?;
                        let minutes = TimeDelta::try_minutes(i64::from(n.minute()))?;
                        let seconds = TimeDelta::try_seconds(i64::from(n.second()))?;
                        Some(hours + minutes + seconds)
                    }),
                    None => start_time.and_then(|start| end_time.map(|end| end - start)),
                };
                Track::new_with_time(
                    row.get("artist").unwrap_or(&"".to_string()).to_string(),
                    row.get("name").unwrap_or(&"".to_string()).to_string(),
                    start_time,
                    end_time,
                    play_time,
                )
            })
            .collect()
    };

    // Remove consecutive duplicates
    let mut index: usize = 0;
    let mut tracks: Vec<Track> = vec![initial_tracks[0].clone()];
    for track in initial_tracks[1..].iter() {
        let previous_track = &tracks[index];
        if *previous_track == *track {
            // duplicate track -> add playtime to previous and skip
            tracks[index] += track.play_time;
            tracks[index].end_time = track.end_time;
        } else {
            // new track, append to playlist
            tracks.push(track.clone());
            index += 1;
        }
    }
    tracks
}
