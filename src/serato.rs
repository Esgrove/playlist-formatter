use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, Timelike};

use super::playlist::Playlist;
use super::track::Track;
use super::types::{FileFormat, PlaylistType};
use super::utils;

/// Read a Serato CSV playlist file.
pub fn read_serato_csv(path: &Path, data: &[BTreeMap<String, String>]) -> anyhow::Result<Playlist> {
    let (playlist_name, playlist_date) = parse_serato_playlist_info(&data[0]);
    let tracks = parse_serato_tracks_from_data(data, playlist_date);
    let total_duration = utils::get_total_playtime(&tracks);
    let max_artist_length: usize = tracks.iter().map(Track::artist_length).max().unwrap_or(0);
    let max_title_length: usize = tracks.iter().map(Track::title_length).max().unwrap_or(0);
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
    playlist_rows: &[BTreeMap<String, String>],
) -> anyhow::Result<Playlist> {
    let required_fields = ["artist", "name"];
    for field in required_fields {
        if !header.contains_key(field) {
            anyhow::bail!("Serato TXT missing required field: '{}'", field)
        }
    }

    let (playlist_name, playlist_date) = parse_serato_playlist_info(&playlist_rows[0]);
    let name = if playlist_name.is_empty() { name } else { playlist_name };
    let date = if playlist_date.is_none() {
        utils::extract_datetime_from_name(&name)
    } else {
        playlist_date
    };
    let tracks = parse_serato_tracks_from_data(playlist_rows, playlist_date);
    let total_duration = utils::get_total_playtime(&tracks);
    let max_artist_length: usize = tracks.iter().map(Track::artist_length).max().unwrap_or(0);
    let max_title_length: usize = tracks.iter().map(Track::title_length).max().unwrap_or(0);
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
#[must_use]
pub fn parse_serato_playlist_info(data: &BTreeMap<String, String>) -> (String, Option<NaiveDateTime>) {
    let playlist_name = data
        .get("name")
        .map_or_else(String::new, std::string::ToString::to_string);
    // timestamp, for example "10.01.2019, 20.00.00 EET"
    let mut playlist_date = data
        .get("start time")
        .and_then(|time| NaiveDateTime::parse_from_str(time, "%d.%m.%Y, %H.%M.%S %Z").ok());
    if playlist_date.is_none() && !playlist_name.is_empty() {
        playlist_date = utils::extract_datetime_from_name(&playlist_name);
    }
    (playlist_name, playlist_date)
}

/// Parse Serato track data from dictionary
#[must_use]
pub fn parse_serato_tracks_from_data(
    data: &[BTreeMap<String, String>],
    playlist_date: Option<NaiveDateTime>,
) -> Vec<Track> {
    let start_date = playlist_date.unwrap_or_default().date();
    let initial_tracks: Vec<Track> = data
        .iter()
        .skip(1)
        .map(|row| parse_track_with_time_from_row(start_date, row))
        .collect();

    // Remove consecutive duplicates
    let mut deduped_tracks: Vec<Track> = Vec::new();
    for track in initial_tracks {
        if let Some(last_track) = deduped_tracks.last_mut() {
            if *last_track == track {
                // Add playtime of duplicate track to previous
                *last_track += track.play_time;
                last_track.end_time = track.end_time;
                continue;
            }
        }
        deduped_tracks.push(track);
    }

    deduped_tracks
}

#[must_use]
pub fn read_serato_txt_lines(initial_lines: Vec<Vec<String>>) -> Vec<Vec<String>> {
    let header_line: String = initial_lines[0][0].clone();
    let column_names: Vec<String> = header_line
        .replace("     ", "\t")
        .split('\t')
        .filter_map(|s| {
            let v = s.trim();
            if s.is_empty() { None } else { Some(v.to_string()) }
        })
        .collect();

    // Get starting location of each column item on a line
    let mut column_start_indices: Vec<usize> = column_names
        .iter()
        .filter_map(|field| header_line.find(field))
        .collect();

    // Extract each column item from the line, starting from the end of the line
    column_start_indices.reverse();
    let mut serato_lines: Vec<Vec<String>> = Vec::new();
    for line in initial_lines {
        // skip the divider lines
        if line[0].chars().all(|c| c == '-') {
            continue;
        }
        let mut split_line: Vec<String> = Vec::new();
        let mut remaining_line: String = line[0].to_string();
        for index in &column_start_indices {
            // some lines do not contain data in all fields
            if *index >= remaining_line.len() {
                continue;
            }
            let (string_left, value) = remaining_line.split_at(*index);
            split_line.push(value.trim().to_string());
            remaining_line = string_left.to_string();
        }
        // Convert line item order since we extracted items in reverse order
        split_line.reverse();
        // pad line in case some columns did not have any data
        while split_line.len() < column_names.len() {
            split_line.push(String::new());
        }
        serato_lines.push(split_line);
    }
    serato_lines
}

fn parse_track_with_time_from_row(start_date: NaiveDate, row: &BTreeMap<String, String>) -> Track {
    let start_time: Option<NaiveDateTime> = row
        .get("start time")
        .and_then(|t| NaiveTime::parse_from_str(t, "%H.%M.%S %Z").ok())
        .map(|time| NaiveDateTime::new(start_date, time));

    let end_time: Option<NaiveDateTime> = row
        .get("end time")
        .and_then(|t| NaiveTime::parse_from_str(t, "%H.%M.%S %Z").ok())
        .map(|time| NaiveDateTime::new(start_date, time));

    let play_time = row.get("playtime").map_or_else(
        || start_time.and_then(|start| end_time.map(|end| end - start)),
        |t| {
            NaiveTime::parse_from_str(t, "%H:%M:%S").ok().and_then(|n| {
                let hours = TimeDelta::try_hours(i64::from(n.hour()))?;
                let minutes = TimeDelta::try_minutes(i64::from(n.minute()))?;
                let seconds = TimeDelta::try_seconds(i64::from(n.second()))?;
                Some(hours + minutes + seconds)
            })
        },
    );
    Track::new_with_time(
        row.get("artist").unwrap_or(&String::new()).to_string(),
        row.get("name").unwrap_or(&String::new()).to_string(),
        start_time,
        end_time,
        play_time,
    )
}
