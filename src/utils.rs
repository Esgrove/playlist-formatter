use crate::track::Track;

use anyhow::anyhow;
use anyhow::Result;
use chrono::Duration;
use strum::EnumIter;
use strum_macros::Display;

use std::ffi::OsStr;
use std::ffi::OsString;
use std::path::PathBuf;
use std::str::FromStr;
use std::string::String;

/// Playlist file type
#[derive(Debug, PartialEq, EnumIter, Display)]
pub enum FileFormat {
    Txt,
    Csv,
}

/// Output formatting style for playlist printing
#[derive(Default, Debug, PartialEq, Display)]
pub enum FormattingStyle {
    /// Basic formatting, for example for sharing playlist text online
    Basic,
    /// Basic formatting but with track numbers
    Numbered,
    /// Pretty formatting for human readable formatted CLI output
    #[default]
    Pretty,
}

/// Which DJ software is the playlist from.
///
/// Each software has its own formatting style.
/// Formatted means it was already processed by this program.
#[derive(Debug, PartialEq, Display)]
pub enum PlaylistType {
    Rekordbox,
    Serato,
    Formatted,
}

/// Logging level
#[derive(clap::ValueEnum, Clone, Debug, Display)]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    pub fn to_log_filter(&self) -> log::LevelFilter {
        match self {
            Level::Trace => log::LevelFilter::Trace,
            Level::Debug => log::LevelFilter::Debug,
            Level::Info => log::LevelFilter::Info,
            Level::Warn => log::LevelFilter::Warn,
            Level::Error => log::LevelFilter::Error,
        }
    }
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
pub fn get_total_playtime(tracks: &[Track]) -> Option<Duration> {
    let mut sum = Duration::seconds(0);
    for track in tracks.iter() {
        if let Some(duration) = track.play_time {
            // chrono::Duration does not implement AddAssign or sum() :(
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
pub fn formatted_duration(duration: Duration) -> String {
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

/// Convert string to `FileFormat` enum
impl FromStr for FileFormat {
    type Err = anyhow::Error;
    fn from_str(input: &str) -> Result<FileFormat> {
        match input.to_lowercase().trim() {
            "csv" => Ok(FileFormat::Csv),
            "txt" => Ok(FileFormat::Txt),
            "" => Err(anyhow!("Can't convert empty string to file format")),
            _ => Err(anyhow!("Unsupported file format: '{input}'")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let duration = Duration::seconds(59);
        assert_eq!(formatted_duration(duration), "0:59");

        let duration = Duration::seconds(77);
        assert_eq!(formatted_duration(duration), "1:17");

        let duration = Duration::minutes(45);
        assert_eq!(formatted_duration(duration), "45:00");

        let duration = Duration::minutes(60);
        assert_eq!(formatted_duration(duration), "1:00:00");

        let duration = Duration::minutes(31) + Duration::seconds(33);
        assert_eq!(formatted_duration(duration), "31:33");
    }

    #[test]
    fn test_from_str_valid_format() {
        assert_eq!(FileFormat::from_str("csv").unwrap(), FileFormat::Csv);
        assert_eq!(FileFormat::from_str("txt").unwrap(), FileFormat::Txt);
    }

    #[test]
    fn from_str_unsupported_format() {
        let result = FileFormat::from_str("mp3");
        assert!(
            result.is_err(),
            "Expected an error for unsupported format, but got {:?}",
            result
        );
        if let Err(e) = result {
            assert_eq!(e.to_string(), "Unsupported file format: 'mp3'");
        }
    }

    #[test]
    fn from_str_empty_string() {
        let result = FileFormat::from_str("");
        assert!(
            result.is_err(),
            "Expected an error for empty string, but got {:?}",
            result
        );
        if let Err(e) = result {
            assert_eq!(e.to_string(), "Can't convert empty string to file format");
        }
    }
}
