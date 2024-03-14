use std::ffi::OsStr;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::string::String;

use anyhow::Context;
use anyhow::Result;
use chrono::{NaiveDate, NaiveDateTime, TimeDelta};
use home::home_dir;
use lazy_static::lazy_static;
use regex::Regex;
use strum::IntoEnumIterator;

use crate::track::Track;
use crate::types::FileFormat;

lazy_static! {
    static ref RE_DD_MM_YYYY: Regex =
        Regex::new(r"(\d{1,2})\.(\d{1,2})\.(\d{4})").expect("Failed to create regex pattern for dd.mm.yyyy");
    static ref RE_YYYY_MM_DD: Regex =
        Regex::new(r"(\d{4})\.(\d{1,2})\.(\d{1,2})").expect("Failed to create regex pattern for yyyy.mm.dd");
}

/// Append extension to `PathBuf`, which is somehow missing completely from the standard lib :(
///
/// <https://internals.rust-lang.org/t/pathbuf-has-set-extension-but-no-add-extension-cannot-cleanly-turn-tar-to-tar-gz/14187/10>
pub fn append_extension_to_path(path: PathBuf, extension: impl AsRef<OsStr>) -> PathBuf {
    let mut os_string: OsString = path.into();
    os_string.push(".");
    os_string.push(extension);
    os_string.into()
}

/// Get total playtime for a list of tracks
pub fn get_total_playtime(tracks: &[Track]) -> Option<TimeDelta> {
    let mut sum = TimeDelta::try_seconds(0)?;
    for track in tracks.iter() {
        if let Some(duration) = track.play_time {
            sum += duration;
        }
    }
    if sum.is_zero() {
        None
    } else {
        Some(sum)
    }
}

/// Format duration as a string either as H:MM:SS or MM:SS depending on the duration.
pub fn formatted_duration(duration: TimeDelta) -> String {
    let hours = duration.num_hours();
    let minutes = duration.num_minutes();
    let seconds = duration.num_seconds();
    if seconds > 0 {
        if minutes >= 60 {
            format!("{}:{:02}:{:02}", hours, minutes % 60, seconds % 60)
        } else {
            format!("{}:{:02}", minutes, seconds % 60)
        }
    } else {
        String::new()
    }
}

pub fn extract_datetime_from_name(input: &str) -> Option<NaiveDateTime> {
    if let Some(caps) = RE_DD_MM_YYYY.captures(input) {
        let day = caps.get(1)?.as_str().parse::<u32>().ok()?;
        let month = caps.get(2)?.as_str().parse::<u32>().ok()?;
        let year = caps.get(3)?.as_str().parse::<i32>().ok()?;
        let date = NaiveDate::from_ymd_opt(year, month, day)?;
        return date.and_hms_opt(0, 0, 0);
    }
    if let Some(caps) = RE_YYYY_MM_DD.captures(input) {
        let year = caps.get(1)?.as_str().parse::<i32>().ok()?;
        let month = caps.get(2)?.as_str().parse::<u32>().ok()?;
        let day = caps.get(3)?.as_str().parse::<u32>().ok()?;
        let date = NaiveDate::from_ymd_opt(year, month, day)?;
        return date.and_hms_opt(0, 0, 0);
    }
    None
}

/// Get DJ playlist directory path in Dropbox if it exists
pub fn dropbox_save_dir() -> Option<PathBuf> {
    let path = if cfg!(target_os = "windows") {
        Some(dunce::simplified(Path::new("D:\\Dropbox\\DJ\\PLAYLIST")).to_path_buf())
    } else if let Some(mut home) = home_dir() {
        home.push("Dropbox/DJ/PLAYLIST");
        Some(dunce::simplified(&home).to_path_buf())
    } else {
        None
    };
    path.filter(|p| p.is_dir())
}

/// Get the longest formatted track playtime length in number of chars.
pub fn get_max_playtime_length(tracks: &[Track]) -> usize {
    tracks
        .iter()
        .map(|t| {
            formatted_duration(t.play_time.unwrap_or(TimeDelta::try_seconds(0).unwrap()))
                .chars()
                .count()
        })
        .max()
        .unwrap_or(0)
}

/// Get playlist format enum from the file extension.
pub fn playlist_format(file: &Path) -> Result<FileFormat> {
    let extension: &str = match file.extension() {
        None => {
            anyhow::bail!(
                "Input file has no file extension: '{}'. Supported file types are: {}",
                file.display(),
                FileFormat::iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        }
        Some(ext) => ext.to_str().context("Failed to parse file extension")?,
    };
    FileFormat::from_str(extension)
}

#[cfg(test)]
mod tests {
    use crate::utils::*;

    #[test]
    fn test_append_extension_to_path() {
        let path = PathBuf::from("/path/to/file");
        let result = append_extension_to_path(path, "csv");
        assert_eq!(result.to_str().unwrap(), "/path/to/file.csv");

        let path = PathBuf::from("/path/to/file-2024.01.01");
        let result = append_extension_to_path(path, "csv");
        assert_eq!(result.to_str().unwrap(), "/path/to/file-2024.01.01.csv");

        let path = PathBuf::from("14.5.2001");
        let result = append_extension_to_path(path, "txt");
        assert_eq!(result.to_str().unwrap(), "14.5.2001.txt");
    }

    #[test]
    fn test_get_total_playtime() {
        let tracks: Vec<Track> = Vec::new();
        assert_eq!(get_total_playtime(&tracks), None);
    }

    #[test]
    fn test_formatted_duration() {
        let duration = TimeDelta::try_seconds(59).unwrap();
        assert_eq!(formatted_duration(duration), "0:59");

        let duration = TimeDelta::try_seconds(77).unwrap();
        assert_eq!(formatted_duration(duration), "1:17");

        let duration = TimeDelta::try_minutes(45).unwrap();
        assert_eq!(formatted_duration(duration), "45:00");

        let duration = TimeDelta::try_minutes(60).unwrap();
        assert_eq!(formatted_duration(duration), "1:00:00");

        let duration = TimeDelta::try_minutes(31).unwrap() + TimeDelta::try_seconds(33).unwrap();
        assert_eq!(formatted_duration(duration), "31:33");
    }
}
