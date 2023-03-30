use anyhow::anyhow;
use anyhow::{Context, Result};
use chrono::{Duration, NaiveDateTime, NaiveTime, Timelike};
use colored::Colorize;
use csv::Reader;
use encoding_rs_io::DecodeReaderBytes;
use home::home_dir;
use std::cmp::max;
use std::collections::{BTreeMap, HashMap};
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs::File;
use std::io::{Read, Write};
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

/// Represents one played track
#[derive(Debug, Clone)]
struct Track {
    artist: String,
    title: String,
    start_time: Option<NaiveDateTime>,
    end_time: Option<NaiveDateTime>,
    play_time: Option<Duration>,
}

/// Parsed playlist data
#[derive(Debug)]
pub(crate) struct Playlist {
    pub date: Option<NaiveDateTime>,
    pub file_format: FileFormat,
    pub file: PathBuf,
    pub name: String,
    pub playlist_type: PlaylistType,
    pub total_duration: Option<Duration>,
    tracks: Vec<Track>,
    max_artist_length: usize,
    max_title_length: usize,
    max_playtime_length: usize,
}

impl Track {
    /// Create a simple track with only artist name and song title
    pub fn new(artist: String, title: String) -> Track {
        Track {
            artist,
            title,
            start_time: None,
            end_time: None,
            play_time: None,
        }
    }

    /// Create a track with full information including start and play time.
    pub fn new_with_time(
        artist: String,
        title: String,
        start_time: Option<NaiveDateTime>,
        end_time: Option<NaiveDateTime>,
        play_time: Option<Duration>,
    ) -> Track {
        Track {
            artist,
            title,
            start_time,
            end_time,
            play_time,
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
            FileFormat::Csv => Self::read_csv(file).unwrap(),
            FileFormat::Txt => Self::read_txt(file).unwrap(),
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
                .map(|row| {
                    Track::new(
                        row.get("Artist").unwrap().to_string(),
                        row.get("Track Title").unwrap().to_string(),
                    )
                })
                .collect()
        };

        // Remove consecutive duplicates
        // TODO: handle play times
        tracks.dedup();

        let total_duration = get_total_playtime(&tracks);

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
        let max_playtime_length: usize = max(
            "PLAYTIME".to_string().chars().count(),
            tracks
                .iter()
                .map(|t| {
                    formatted_duration(t.play_time.unwrap_or(Duration::seconds(0)))
                        .chars()
                        .count()
                })
                .max()
                .unwrap_or(0),
        );

        Ok(Playlist {
            date: None,
            file: PathBuf::from(path),
            file_format: FileFormat::Txt,
            name,
            playlist_type: PlaylistType::Rekordbox,
            tracks,
            max_artist_length,
            max_title_length,
            max_playtime_length,
            total_duration,
        })
    }

    /// Read a .csv playlist file
    fn read_csv(path: &Path) -> Result<Playlist> {
        let mut reader = Reader::from_path(path)
            .with_context(|| format!("Failed to open CSV file: '{}'", path.display()))?;

        // map each header name to the column index they correspond to in the data, for example:
        // {"name": 0, "artist": 1, "start time": 2}
        let header_map: BTreeMap<String, usize> = {
            let headers = reader.headers()?;
            headers
                .iter()
                .enumerate()
                .map(|(index, value)| (value.to_string(), index))
                .collect()
        };

        log::debug!(
            "CSV headers ({}): {:?}",
            header_map.keys().len(),
            header_map.keys()
        );

        let required_fields = vec!["name", "artist"];
        for field in required_fields {
            if !header_map.contains_key(field) {
                anyhow::bail!("CSV missing required field: {}", field)
            }
        }

        // Map track data to a dictionary (header key: value)
        let data: Vec<BTreeMap<String, String>> = {
            reader
                .records()
                .map(|s| {
                    let record = s.unwrap();
                    let mut items: BTreeMap<String, String> = BTreeMap::new();
                    for (name, index) in &header_map {
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
        let playlist_date = NaiveDateTime::parse_from_str(
            data[0].get("start time").unwrap(),
            "%d.%m.%Y, %H.%M.%S %Z",
        )
        .unwrap();

        // parse tracks
        let initial_tracks: Vec<Track> = {
            data[1..]
                .iter()
                .map(|row| {
                    let start_time: Option<NaiveDateTime> = {
                        match row.get("start time") {
                            None => None,
                            Some(t) => match NaiveTime::parse_from_str(t, "%H.%M.%S %Z") {
                                Ok(n) => Some(NaiveDateTime::new(playlist_date.date(), n)),
                                Err(_) => None,
                            },
                        }
                    };
                    let end_time: Option<NaiveDateTime> = {
                        match row.get("end time") {
                            None => None,
                            Some(t) => match NaiveTime::parse_from_str(t, "%H.%M.%S %Z") {
                                Ok(n) => Some(NaiveDateTime::new(playlist_date.date(), n)),
                                Err(_) => None,
                            },
                        }
                    };
                    let play_time: Option<Duration> = {
                        match row.get("playtime") {
                            None => None,
                            Some(t) => match NaiveTime::parse_from_str(t, "%H:%M:%S") {
                                Ok(n) => Some(
                                    Duration::hours(n.hour() as i64)
                                        + Duration::minutes(n.minute() as i64)
                                        + Duration::seconds(n.second() as i64),
                                ),
                                Err(_) => None,
                            },
                        }
                    };
                    Track::new_with_time(
                        row.get("artist").unwrap().to_string(),
                        row.get("name").unwrap().to_string(),
                        start_time,
                        end_time,
                        play_time,
                    )
                })
                .collect()
        };

        let mut new_tracks_index: usize = 0;
        let mut tracks: Vec<Track> = vec![initial_tracks[0].clone()];
        for track in initial_tracks[1..].iter() {
            let previous_track = &tracks[new_tracks_index];
            if *previous_track == *track {
                // duplicate track -> add playtime to previous
                tracks[new_tracks_index] = previous_track.clone() + track.play_time
            } else {
                // new track, append to playlist
                tracks.push(track.clone());
                new_tracks_index += 1;
            }
        }

        // Remove consecutive duplicates
        // TODO: handle play times
        tracks.dedup();

        let total_duration = get_total_playtime(&tracks);

        let max_artist_length: usize = tracks.iter().map(|t| t.artist_length()).max().unwrap_or(0);
        let max_title_length: usize = tracks.iter().map(|t| t.title_length()).max().unwrap_or(0);
        let max_playtime_length: usize = max(
            "PLAYTIME".to_string().chars().count(),
            tracks
                .iter()
                .map(|t| {
                    formatted_duration(t.play_time.unwrap_or(Duration::seconds(0)))
                        .chars()
                        .count()
                })
                .max()
                .unwrap_or(0),
        );

        Ok(Playlist {
            date: Some(playlist_date),
            file: PathBuf::from(path),
            file_format: FileFormat::Csv,
            name: playlist_name.clone(),
            playlist_type: PlaylistType::Serato,
            tracks,
            max_artist_length,
            max_title_length,
            max_playtime_length,
            total_duration,
        })
    }

    /// Print playlist information (but not the tracks themselves)
    pub fn print_info(&self) {
        println!("Playlist: {}", self.name.green(),);
        println!("Filepath: {}", self.file.canonicalize().unwrap().display());
        println!(
            "Format: {}, Type: {}, Date: {}",
            self.file_format,
            self.playlist_type.to_string().cyan(),
            if let Some(date) = self.date {
                date.format("%Y.%m.%d %H:%M").to_string()
            } else {
                "unknown".to_string()
            }
        );
        print!("Tracks: {}", self.tracks.len());
        if let Some(duration) = self.total_duration {
            print!(", Total duration: {}", formatted_duration(duration));
            let average = Duration::seconds(duration.num_seconds() / self.tracks.len() as i64);
            print!(" ({} per track)", formatted_duration(average))
        };
        println!("\n");
    }

    /// Print playlist with the given formatting style
    pub fn print_playlist(&self, style: FormattingStyle) {
        match style {
            FormattingStyle::Basic => self.print_simple_playlist(),
            FormattingStyle::Numbered => self.print_numbered_playlist(),
            FormattingStyle::Pretty => self.print_pretty_playlist(),
        }
    }

    /// Write playlist to file.
    ///
    /// File path and type will be parsed from the cli option if present.
    /// Otherwise, will try to use default path and file format.
    pub fn save_playlist_to_file(
        &self,
        filepath: Option<String>,
        overwrite_existing: bool,
    ) -> Result<()> {
        let potential_path: Option<PathBuf> = match filepath {
            Some(value) => {
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(trimmed))
                }
            }
            None => None,
        };

        let path = if let Some(value) = potential_path {
            log::info!("Got output path: {}", value.display());
            // Possible options here:
            // 1. full path to file
            // 2. file name with extension
            // 3. file name without extension
            // ->
            // check if there is a file extension, and add default if not
            // check if it is a path
            let has_valid_file_extension = match value.extension() {
                Some(extension) => {
                    // check if this is a supported file format
                    FileFormat::from_str(extension.to_str().unwrap()).is_ok()
                }
                None => false,
            };
            if !has_valid_file_extension {
                // Can't use `with_extension` here since it will replace anything after the last dot,
                // which will alter the name if it contains a date separated by dots for example.
                append_extension_to_pathbuf(value, "csv")
            } else {
                value
            }
        } else {
            log::debug!("Empty output path given, using default...");
            let mut save_dir: PathBuf = self.default_save_dir();

            log::info!("Using default save dir: {}", save_dir.display());
            save_dir.push(self.file.with_extension("csv"));
            save_dir
        };

        log::info!("Saving to: {}", path.display());

        if path.is_file() {
            log::info!("Output file already exists: {}", path.display());
            if !overwrite_existing {
                anyhow::bail!(
                    "use the {} option overwrite an existing output file",
                    "force".bold()
                );
            } else {
                log::info!("Overwriting existing file")
            }
        }

        self.write_playlist_file(path.as_path())
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
            "{:<index_width$}   {:<artist_width$}   {:<title_width$}   {:>playtime_width$}",
            "#",
            "ARTIST",
            "TITLE",
            "PLAYTIME",
            index_width = index_width,
            artist_width = self.max_artist_length,
            title_width = self.max_title_length,
            playtime_width = self.max_playtime_length
        );

        let header_width = header.chars().count();
        let divider = "-".repeat(header_width);

        println!("{}", header.bold());
        println!("{divider}");

        for (index, track) in self.tracks.iter().enumerate() {
            println!(
                "{:>0index_width$}   {:<artist_width$}   {:<title_width$}   {:>playtime_width$}",
                index + 1,
                track.artist,
                track.title,
                if let Some(d) = track.play_time {
                    formatted_duration(d)
                } else {
                    "".to_string()
                },
                index_width = index_width,
                artist_width = self.max_artist_length,
                title_width = self.max_title_length,
                playtime_width = self.max_playtime_length
            );
        }

        println!("{divider}");
    }

    /// Get playlist format enum from file extension
    fn playlist_format(file: &Path) -> FileFormat {
        let extension = file.extension().unwrap().to_str().unwrap();
        FileFormat::from_str(extension).unwrap()
    }

    /// Return default save directory for playlist output file.
    ///
    /// This will first try to use the Dropbox playlist directory if it exists on disk.
    /// After that, it will try the get the directory of the input file.
    /// Otherwise returns an empty path so the file will go to the current working directory.
    fn default_save_dir(&self) -> PathBuf {
        if let Some(dir) = Playlist::dropbox_save_dir() {
            dir
        } else {
            let default_dir = match self.file.canonicalize() {
                Ok(path) => match path.parent() {
                    Some(parent) => parent.to_path_buf(),
                    None => PathBuf::new(),
                },
                Err(error) => {
                    log::error!("Failed to resolve full path to input file: {}", error);
                    PathBuf::new()
                }
            };
            default_dir
        }
    }

    /// Write playlist to file. Filepath needs to be a ".txt" or ".csv" file.
    fn write_playlist_file(&self, filepath: &Path) -> Result<()> {
        match filepath
            .extension()
            .unwrap()
            .to_str()
            .unwrap()
            .to_lowercase()
            .as_str()
        {
            "csv" => self.write_csv_file(filepath),
            "txt" => self.write_txt_file(filepath),
            _ => anyhow::bail!("Unsupported file extension"),
        }
    }

    /// Write tracks to CSV file
    fn write_csv_file(&self, filepath: &Path) -> Result<()> {
        let mut writer = csv::Writer::from_path(filepath)?;
        writer.write_record(["artist", "", "title"])?;
        for track in self.tracks.iter() {
            writer.write_record([track.artist.clone(), "-".to_string(), track.title.clone()])?;
        }
        writer.flush()?;
        Ok(())
    }

    /// Write tracks to TXT file
    fn write_txt_file(&self, filepath: &Path) -> Result<()> {
        let mut file = File::create(filepath)?;
        for track in self.tracks.iter() {
            file.write_all(format!("{} - {}\n", track.artist, track.title).as_ref())?;
        }
        Ok(())
    }

    /// Get DJ playlist path in Dropbox if it exists
    fn dropbox_save_dir() -> Option<PathBuf> {
        let path = if cfg!(target_os = "windows") {
            Some(PathBuf::from("D:\\Dropbox\\DJ\\PLAYLIST"))
        } else if let Some(mut home) = home_dir() {
            home.push("Dropbox/DJ/PLAYLIST");
            Some(home)
        } else {
            None
        };
        path.filter(|p| p.is_dir())
    }
}

/// Get total playtime for a list of tracks
fn get_total_playtime(tracks: &[Track]) -> Option<Duration> {
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
fn formatted_duration(duration: Duration) -> String {
    let hours = duration.num_hours();
    let minutes = duration.num_minutes();
    let seconds = duration.num_seconds();
    if seconds > 0 {
        if minutes >= 60 {
            return format!("{}:{:02}:{:02}", hours, minutes % 60, seconds % 60);
        } else {
            return format!("{}:{:02}", minutes, seconds % 60);
        }
    }
    "".to_string()
}

/// Append extension to PathBuf, which is somehow missing completely from the standard lib :(
///
/// https://internals.rust-lang.org/t/pathbuf-has-set-extension-but-no-add-extension-cannot-cleanly-turn-tar-to-tar-gz/14187/10
fn append_extension_to_pathbuf(path: PathBuf, extension: impl AsRef<OsStr>) -> PathBuf {
    let mut os_string: OsString = path.into();
    os_string.push(".");
    os_string.push(extension.as_ref());
    os_string.into()
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
            end_time: self.end_time,
            play_time: if let Some(time) = self.play_time {
                Some(time + duration)
            } else {
                Some(duration)
            },
        }
    }
}

impl ops::Add<Option<Duration>> for Track {
    type Output = Track;
    fn add(self, duration: Option<Duration>) -> Track {
        let play_time = match self.play_time {
            Some(time) => match duration {
                None => Some(time),
                Some(d) => Some(time + d),
            },
            None => duration,
        };
        Track {
            artist: self.artist,
            title: self.title,
            start_time: self.start_time,
            end_time: self.end_time,
            play_time,
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

impl fmt::Display for Track {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {}", self.artist, self.title)
    }
}
