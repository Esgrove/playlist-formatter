use anyhow::anyhow;
use anyhow::{Context, Result};
use chrono::{Duration, NaiveDateTime, NaiveTime};
use csv::Reader;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::string::String;

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
    artist: String,
    title: String,
    start_time: Option<NaiveDateTime>,
    play_time: Option<Duration>,
}

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
    pub fn new(file: &Path) -> Playlist {
        let format = Self::playlist_format(file);
        match format {
            PlaylistFormat::Csv => Self::read_csv(file).unwrap(),
            PlaylistFormat::Txt => Self::read_txt(file).unwrap(),
        }
    }

    fn read_txt(file: &Path) -> Result<Playlist> {
        Ok(Playlist {
            date: Default::default(),
            file: PathBuf::from(file),
            format: PlaylistFormat::Txt,
            name: file.file_name().unwrap().to_str().unwrap().to_string(),
            playlist_type: PlaylistType::Rekordbox,
            tracks: vec![],
        })
    }

    fn read_csv(file: &Path) -> Result<Playlist> {
        let mut reader = Reader::from_path(file)
            .with_context(|| format!("Failed to open CSV file: '{}'", file.display()))?;

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
            file: PathBuf::from(file),
            format: PlaylistFormat::Csv,
            name: playlist_name.clone(),
            playlist_type: PlaylistType::Serato,
            tracks,
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
