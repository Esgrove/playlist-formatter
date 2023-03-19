use anyhow::anyhow;
use anyhow::{Context, Result};
use chrono::{Duration, NaiveDateTime, NaiveTime};
use csv::Reader;
use encoding_rs_io::DecodeReaderBytes;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::fs::File;
use std::io::BufRead;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::string::String;

/// Which DJ software is the playlist from.
/// Each software has their own formatting style.
#[derive(Debug, PartialEq)]
enum PlaylistType {
    Rekordbox,
    Serato,
}

/// Playlist file type
#[derive(Debug, PartialEq)]
enum PlaylistFormat {
    Txt,
    Csv,
}

/// Represents one played track
#[derive(Debug)]
struct Track {
    artist: String,
    title: String,
    start_time: Option<NaiveDateTime>,
    play_time: Option<Duration>,
}

/// Parsed playlist data
#[derive(Debug)]
pub(crate) struct Playlist {
    date: NaiveDateTime,
    file: PathBuf,
    format: PlaylistFormat,
    name: String,
    playlist_type: PlaylistType,
    tracks: Vec<Track>,
}

impl Playlist {
    /// Initialize playlist from given filepath
    pub fn new(file: &Path) -> Playlist {
        let format = Self::playlist_format(file);
        match format {
            PlaylistFormat::Csv => Self::read_csv(file).unwrap(),
            PlaylistFormat::Txt => Self::read_txt(file).unwrap(),
        }
    }

    /// Read a .txt playlist file
    fn read_txt(path: &Path) -> Result<Playlist> {
        let file = File::open(path)?;
        // Rekordbox encodes txt files in UTF-16 :(
        // This implementation is far from ideal since it reads everything into a single string
        let mut decoder = DecodeReaderBytes::new(file);
        let mut dest = String::new();
        decoder.read_to_string(&mut dest)?;

        let lines: Vec<Vec<String>> = dest
            .lines()
            .map(|s| s.split('\t').map(|l| l.trim().to_string()).collect())
            .collect();

        log::debug!("Lines ({}):", lines.len());
        log::debug!("{:#?}", lines);

        // map each header name to the row index they correspond to in the data, for example:
        // {"name": 0, "artist": 1, "start time": 2}
        let map: HashMap<String, usize> = {
            let headers = &lines[0];
            headers
                .iter()
                .enumerate()
                .map(|(index, value)| (value.to_string(), index))
                .collect()
        };

        log::debug!("txt headers ({}): {:?}", map.keys().len(), map.keys());

        let data: Vec<BTreeMap<String, String>> = {
            lines[1..]
                .iter()
                .map(|line| {
                    let mut items: BTreeMap<String, String> = BTreeMap::new();
                    for (name, index) in &map {
                        let value = &line[*index];
                        items.insert(name.to_string(), value.to_string());
                    }
                    items
                })
                .collect()
        };

        log::debug!("Rows ({}):", data.len());
        for row in &data {
            log::debug!("{:#?}", row);
        }

        let tracks: Vec<Track> = {
            data.iter()
                .map(|row| Track {
                    title: row.get("Track Title").unwrap().to_string(),
                    artist: row.get("Artist").unwrap().to_string(),
                    start_time: None,
                    play_time: None,
                })
                .collect()
        };

        Ok(Playlist {
            date: NaiveDateTime::default(),
            file: PathBuf::from(path),
            format: PlaylistFormat::Txt,
            name: path.file_name().unwrap().to_str().unwrap().to_string(),
            playlist_type: PlaylistType::Rekordbox,
            tracks,
        })
    }

    /// Read a .csv playlist file
    fn read_csv(path: &Path) -> Result<Playlist> {
        let mut reader = Reader::from_path(path)
            .with_context(|| format!("Failed to open CSV file: '{}'", path.display()))?;

        // map each header name to the row index they correspond to in the data, for example:
        // {"name": 0, "artist": 1, "start time": 2}
        let map: HashMap<String, usize> = {
            let headers = reader.headers()?;
            headers
                .iter()
                .enumerate()
                .map(|(index, value)| (value.to_string(), index))
                .collect()
        };

        log::debug!("CSV headers ({}): {:?}", map.keys().len(), map.keys());

        let required_fields = vec!["name", "artist"];
        for field in required_fields {
            if !map.contains_key(field) {
                anyhow::bail!("CSV missing required field: {}", field)
            }
        }

        let data: Vec<BTreeMap<String, String>> = {
            reader
                .records()
                .map(|s| {
                    let record = s.unwrap();
                    let mut items: BTreeMap<String, String> = BTreeMap::new();
                    for (name, index) in &map {
                        let value = &record[*index];
                        items.insert(name.to_string(), value.to_string());
                    }
                    items
                })
                .collect()
        };

        log::debug!("Rows ({}):", data.len());
        for row in &data {
            log::debug!("{:?}", row);
        }

        // first row in Serato CSV is an info row with the playlist name and timestamp
        let playlist_name = &data[0].get("name").unwrap().to_string();
        // timestamp, for example "10.01.2019, 20.00.00 EET"
        let playlist_time = NaiveDateTime::parse_from_str(
            data[0].get("start time").unwrap(),
            "%d.%m.%Y, %H.%M.%S %Z",
        )
        .unwrap();

        // parse tracks
        let tracks: Vec<Track> = {
            data[1..]
                .iter()
                .map(|row| {
                    let start_time: Option<NaiveDateTime> = {
                        match row.get("start time") {
                            None => None,
                            Some(t) => match NaiveTime::parse_from_str(t, "%H.%M.%S %Z") {
                                Ok(n) => Some(NaiveDateTime::new(playlist_time.date(), n)),
                                Err(_) => None,
                            },
                        }
                    };
                    Track {
                        title: row.get("name").unwrap().to_string(),
                        artist: row.get("artist").unwrap().to_string(),
                        start_time,
                        play_time: None,
                    }
                })
                .collect()
        };

        Ok(Playlist {
            date: playlist_time,
            file: PathBuf::from(path),
            format: PlaylistFormat::Csv,
            name: playlist_name.clone(),
            playlist_type: PlaylistType::Serato,
            tracks,
        })
    }

    /// Get playlist format enum from file extension
    fn playlist_format(file: &Path) -> PlaylistFormat {
        let extension = file.extension().unwrap().to_str().unwrap();
        PlaylistFormat::from_str(extension).unwrap()
    }
}

/// Convert string to enum
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
