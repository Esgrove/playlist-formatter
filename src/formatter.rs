use anyhow::anyhow;
use anyhow::{Context, Result};
use chrono::{Duration, NaiveDateTime, NaiveTime};
use colored::Colorize;
use csv::Reader;
use encoding_rs_io::DecodeReaderBytes;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::BufRead;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::string::String;
use std::{fmt, ops};

/// Which DJ software is the playlist from.
/// Each software has their own formatting style.
#[derive(Debug, PartialEq)]
pub enum PlaylistType {
    Rekordbox,
    Serato,
}

/// Playlist file type
#[derive(Debug, PartialEq)]
pub enum PlaylistFormat {
    Txt,
    Csv,
}

/// Output formatting style for playlist printing
#[derive(Default, Debug)]
pub enum FormattingStyle {
    /// Simple formatting, for example for sharing playlist text online
    Simple,
    /// Simple but with track numbers
    Numbered,
    /// Pretty formatting for human readable formatted CLI output
    #[default]
    Pretty,
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
    pub date: NaiveDateTime,
    pub file: PathBuf,
    pub format: PlaylistFormat,
    pub name: String,
    pub playlist_type: PlaylistType,
    tracks: Vec<Track>,
    max_artist_length: usize,
    max_title_length: usize,
}

impl Track {
    /// Create a simple track with only artist name and song title
    pub fn new(artist: String, title: String) -> Track {
        Track {
            artist,
            title,
            start_time: None,
            play_time: None,
        }
    }

    /// Create a track with full information including start and play time.
    pub fn new_with_time(
        artist: String,
        title: String,
        start_time: NaiveDateTime,
        play_time: Duration,
    ) -> Track {
        Track {
            artist,
            title,
            start_time: Some(start_time),
            play_time: Some(play_time),
        }
    }

    /// Get the number of characters the artist name has
    pub fn artist_length(&self) -> usize {
        // .len() counts bytes, not chars
        self.artist.chars().count()
    }

    /// Get the number of characters the song title has
    pub fn title_length(&self) -> usize {
        self.title.chars().count()
    }

    // Support summing to increase play time
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

        // Split string to lines, and each line to separate items
        let lines: Vec<Vec<String>> = dest
            .lines()
            .map(|s| s.split('\t').map(|l| l.trim().to_string()).collect())
            .collect();

        log::debug!("Lines ({}):", lines.len());
        log::debug!("{:#?}", lines);

        // Rekordbox txt: first line contains headers.
        // Map each header name to the column index they correspond to in the data, for example:
        // {"#": 0, "Artist": 1, "Track Title": 2}
        let map: HashMap<String, usize> = {
            let headers = &lines[0];
            headers
                .iter()
                .enumerate()
                .map(|(index, value)| (value.to_string(), index))
                .collect()
        };

        log::debug!("txt headers ({}): {:?}", map.keys().len(), map.keys());

        let required_fields = vec!["Track Title", "Artist"];
        for field in required_fields {
            if !map.contains_key(field) {
                anyhow::bail!("TXT missing required field: {}", field)
            }
        }

        // Map track data to a dictionary
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

        let mut tracks: Vec<Track> = {
            data.iter()
                .map(|row| Track {
                    title: row.get("Track Title").unwrap().to_string(),
                    artist: row.get("Artist").unwrap().to_string(),
                    start_time: None,
                    play_time: None,
                })
                .collect()
        };

        // Remove consecutive duplicates
        // TODO: handle play times
        tracks.dedup();

        // Drop file extension from file name
        let name = path
            .with_extension("")
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let max_artist_length: usize = tracks.iter().map(|t| t.artist_length()).max().unwrap_or(0);
        let max_title_length: usize = tracks.iter().map(|t| t.title_length()).max().unwrap_or(0);

        Ok(Playlist {
            date: NaiveDateTime::default(),
            file: PathBuf::from(path),
            format: PlaylistFormat::Txt,
            name,
            playlist_type: PlaylistType::Rekordbox,
            tracks,
            max_artist_length,
            max_title_length,
        })
    }

    /// Read a .csv playlist file
    fn read_csv(path: &Path) -> Result<Playlist> {
        let mut reader = Reader::from_path(path)
            .with_context(|| format!("Failed to open CSV file: '{}'", path.display()))?;

        // map each header name to the column index they correspond to in the data, for example:
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

        // Map track data to a dictionary
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
        let mut tracks: Vec<Track> = {
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

        // Remove consecutive duplicates
        // TODO: handle play times
        tracks.dedup();

        let max_artist_length: usize = tracks.iter().map(|t| t.artist_length()).max().unwrap_or(0);
        let max_title_length: usize = tracks.iter().map(|t| t.title_length()).max().unwrap_or(0);

        Ok(Playlist {
            date: playlist_time,
            file: PathBuf::from(path),
            format: PlaylistFormat::Csv,
            name: playlist_name.clone(),
            playlist_type: PlaylistType::Serato,
            tracks,
            max_artist_length,
            max_title_length,
        })
    }

    /// Print playlist with the given formatting style
    pub fn print_playlist(&self, style: FormattingStyle) {
        match style {
            FormattingStyle::Simple => self.print_simple_playlist(),
            FormattingStyle::Numbered => self.print_numbered_playlist(),
            FormattingStyle::Pretty => self.print_pretty_playlist(),
        }
    }

    /// Print a simple playlist without any formatting
    fn print_simple_playlist(&self) {
        for track in self.tracks.iter() {
            println!("{}", track);
        }
    }

    /// Print a simple playlist with track numbers
    fn print_numbered_playlist(&self) {
        let index_width = self.tracks.len().to_string().chars().count();
        for (index, track) in self.tracks.iter().enumerate() {
            println!(
                "{:>0index_width$}: {}",
                index + 1,
                track,
                index_width = index_width
            );
        }
    }

    /// Print a nicely formatted playlist
    fn print_pretty_playlist(&self) {
        let index_width = self.tracks.len().to_string().chars().count();

        let header = format!(
            "{:<index_width$}  {:<artist_width$}   {:<title_width$}",
            "#",
            "ARTIST",
            "TITLE",
            index_width = index_width,
            artist_width = self.max_artist_length,
            title_width = self.max_title_length
        );

        let header_width = header.chars().count();
        let divider = "-".repeat(header_width);

        println!("{}", header.bold());
        println!("{divider}");

        for (index, track) in self.tracks.iter().enumerate() {
            println!(
                "{:>0index_width$}: {:<artist_width$} - {:<title_width$}",
                index + 1,
                track.artist,
                track.title,
                index_width = index_width,
                artist_width = self.max_artist_length,
                title_width = self.max_title_length,
            );
        }

        println!("{divider}");
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
        match input.to_lowercase().as_str() {
            "csv" => Ok(PlaylistFormat::Csv),
            "txt" => Ok(PlaylistFormat::Txt),
            _ => Err(anyhow!("Unsupported file format: '{input}'")),
        }
    }
}

impl PartialEq for Track {
    fn eq(&self, other: &Self) -> bool {
        self.artist == other.artist && self.title == other.title
    }
}

/// Add duration to play time
impl ops::Add<Duration> for Track {
    type Output = Track;
    fn add(self, duration: Duration) -> Track {
        Track {
            artist: self.artist,
            title: self.title,
            start_time: self.start_time,
            play_time: if let Some(time) = self.play_time {
                Some(time + duration)
            } else {
                Some(duration)
            },
        }
    }
}

/// Add duration to play time
impl ops::AddAssign<Duration> for Track {
    fn add_assign(&mut self, duration: Duration) {
        if let Some(time) = self.play_time {
            self.play_time = Some(time + duration)
        } else {
            self.play_time = Some(duration)
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
        write!(
            f,
            "{}",
            match self {
                PlaylistFormat::Txt => "txt",
                PlaylistFormat::Csv => "csv",
            }
        )
    }
}

impl fmt::Display for Track {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {}", self.artist, self.title)
    }
}
