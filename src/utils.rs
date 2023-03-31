use crate::track::Track;

use anyhow::anyhow;
use anyhow::Result;
use chrono::Duration;
use strum::EnumIter;

use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
use std::string::String;

/// Which DJ software is the playlist from.
/// Each software has their own formatting style.
#[derive(Debug, PartialEq)]
pub enum PlaylistType {
    Rekordbox,
    Serato,
}

/// Playlist file type
#[derive(Debug, PartialEq, EnumIter)]
pub enum FileFormat {
    Txt,
    Csv,
}

/// Output formatting style for playlist printing
#[derive(Default, Debug, PartialEq)]
pub enum FormattingStyle {
    /// Basic formatting, for example for sharing playlist text online
    Basic,
    /// Basic formatting but with track numbers
    Numbered,
    /// Pretty formatting for human readable formatted CLI output
    #[default]
    Pretty,
}

impl fmt::Display for PlaylistType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for FileFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FileFormat::Txt => "txt",
                FileFormat::Csv => "csv",
            }
        )
    }
}

impl fmt::Display for FormattingStyle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FormattingStyle::Basic => "basic",
                FormattingStyle::Numbered => "numbered",
                FormattingStyle::Pretty => "pretty",
            }
        )
    }
}

/// Convert string to enum
impl FromStr for FileFormat {
    type Err = anyhow::Error;
    fn from_str(input: &str) -> Result<FileFormat> {
        match input.to_lowercase().as_str() {
            "csv" => Ok(FileFormat::Csv),
            "txt" => Ok(FileFormat::Txt),
            _ => Err(anyhow!("Unsupported file format: '{input}'")),
        }
    }
}

/// Append extension to PathBuf, which is somehow missing completely from the standard lib :(
///
/// https://internals.rust-lang.org/t/pathbuf-has-set-extension-but-no-add-extension-cannot-cleanly-turn-tar-to-tar-gz/14187/10
pub fn append_extension_to_pathbuf(path: PathBuf, extension: impl AsRef<OsStr>) -> PathBuf {
    let mut os_string: OsString = path.into();
    os_string.push(".");
    os_string.push(extension.as_ref());
    os_string.into()
}

/// Get total playtime for a list of tracks
pub fn get_total_playtime(tracks: &[Track]) -> Option<Duration> {
    let mut sum = Duration::seconds(0);
    for track in tracks.iter() {
        if let Some(duration) = track.play_time {
            // chrono::Duration does not implement AddAssign or sum :(
            sum = sum + duration;
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
        "".to_string()
    }
}
