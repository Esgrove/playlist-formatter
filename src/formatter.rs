use anyhow::anyhow;
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use csv::Reader;
use std::fmt;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug, PartialEq)]
enum PlaylistType {
    Rekordbox,
    Serato,
}

#[derive(Debug, PartialEq)]
enum PlaylistFormat {
    Txt,
    Csv,
}

#[derive(Debug)]
struct Track {
    title: String,
    artist: String,
    start_time: Option<DateTime<Utc>>,
    play_time: Option<Duration>,
}

#[derive(Debug)]
pub(crate) struct Playlist {
    date: DateTime<Utc>,
    file: PathBuf,
    format: PlaylistFormat,
    name: String,
    playlist_type: PlaylistType,
    tracks: Vec<Track>,
}

impl Playlist {
    pub fn new(file: PathBuf) -> Playlist {
        let format = Self::playlist_format(&file);
        match format {
            PlaylistFormat::Csv => Self::read_csv(file)?,
            PlaylistFormat::Txt => Self::read_txt(file)?,
        }
    }

    fn read_txt(file: PathBuf) -> Result<Playlist> {
        Ok(Playlist {
            date: Default::default(),
            file,
            format: PlaylistFormat::Txt,
            name: file.file_name().unwrap().to_str().unwrap().to_string(),
            playlist_type: PlaylistType::Rekordbox,
            tracks: vec![],
        })
    }

    fn read_csv(file: PathBuf) -> Result<Playlist> {
        let mut reader = Reader::from_path(file)
            .with_context(|| format!("Failed to open CSV file: '{}'", file.display()))?;

        {
            let headers = reader.headers()?;
            println!("Headers: {:#?}", headers);
        }

        for result in reader.records() {
            let record = result?;
            println!("{:?}", record);
        }

        Ok(Playlist {
            date: Default::default(),
            file,
            format: PlaylistFormat::Csv,
            name: file.file_name().unwrap().to_str().unwrap().to_string(),
            playlist_type: PlaylistType::Serato,
            tracks: vec![],
        })
    }

    fn playlist_format(file: &Path) -> PlaylistFormat {
        let extension = file.extension().unwrap().to_str().unwrap();
        PlaylistFormat::from_str(extension).unwrap()
    }
}

impl FromStr for PlaylistFormat {
    type Err = anyhow::Error;
    fn from_str(input: &str) -> Result<PlaylistFormat> {
        match input {
            "csv" => Ok(PlaylistFormat::Csv),
            "txt" => Ok(PlaylistFormat::Txt),
            _ => Err(anyhow!("Unsupported file format: '{input}'")),
        }
    }
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
