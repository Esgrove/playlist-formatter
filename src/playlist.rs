use crate::track::Track;
use crate::utils;
use crate::utils::{FileFormat, FormattingStyle, PlaylistType};

use anyhow::{Context, Result};
use chrono::{Duration, NaiveDateTime, NaiveTime, Timelike};
use colored::Colorize;
use csv::Reader;
use encoding_rs_io::DecodeReaderBytes;
use home::home_dir;
use strum::IntoEnumIterator;

use std::cmp::max;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::string::String;

/// Holds imported playlist data
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

impl Playlist {
    /// Initialize playlist from given filepath
    pub fn new(file: &Path) -> Result<Playlist> {
        let format = Self::playlist_format(file)?;
        match format {
            FileFormat::Csv => Self::read_csv(file),
            FileFormat::Txt => Self::read_txt(file),
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

        let lines = Self::read_txt_lines(&mut dest);
        log::debug!("Lines ({}):", lines.len());
        log::debug!("{:#?}", lines);

        // Map each header name to the column index they correspond to in the data, for example:
        // {"#": 0, "Artist": 1, "Track Title": 2}
        let header_map: BTreeMap<String, usize> = {
            let headers = &lines[0];
            headers
                .iter()
                .enumerate()
                .map(|(index, value)| (value.to_string(), index))
                .collect()
        };
        log::debug!(
            "txt headers ({}): {:?}",
            header_map.keys().len(),
            header_map.keys()
        );

        // Map track data to a dictionary
        let data: Vec<BTreeMap<String, String>> = {
            lines[1..]
                .iter()
                .map(|line| {
                    let mut items: BTreeMap<String, String> = BTreeMap::new();
                    for (name, index) in &header_map {
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

        // Drop file extension from file name
        let name = path
            .with_extension("")
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        if header_map.contains_key("name") {
            log::debug!("Detected Serato TXT");
            Playlist::read_serato_txt(path, name, &header_map, &data)
        } else if header_map.contains_key("#") {
            // Rekordbox txt: first line contains headers, line starts with '#'.
            log::debug!("Detected Rekordbox TXT");
            Playlist::read_rekordbox_txt(path, name, &header_map, &data)
        } else {
            anyhow::bail!(
                "Input file does not seem to be a valid Serato or Rekordbox txt playlist"
            );
        }
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
                anyhow::bail!("CSV missing required field: '{}'", field)
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

        let (playlist_name, playlist_date) = Self::parse_serato_playlist_info(&data[0]);
        let tracks = Self::parse_serato_tracks_from_data(&data, playlist_date);
        let total_duration = utils::get_total_playtime(&tracks);
        let max_artist_length: usize = tracks.iter().map(|t| t.artist_length()).max().unwrap_or(0);
        let max_title_length: usize = tracks.iter().map(|t| t.title_length()).max().unwrap_or(0);
        let max_playtime_length: usize = Self::get_max_playtime_length(&tracks);

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

    fn parse_serato_playlist_info(
        data: &BTreeMap<String, String>,
    ) -> (String, Option<NaiveDateTime>) {
        // first row in Serato CSV is an info row with the playlist name and timestamp
        let playlist_name = match data.get("name") {
            None => String::new(),
            Some(n) => n.to_string(),
        };
        // timestamp, for example "10.01.2019, 20.00.00 EET"
        let playlist_date = match data.get("start time") {
            None => None,
            Some(time) => NaiveDateTime::parse_from_str(time, "%d.%m.%Y, %H.%M.%S %Z").ok(),
        };
        (playlist_name, playlist_date)
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
            print!(", Total duration: {}", utils::formatted_duration(duration));
            let average = Duration::seconds(duration.num_seconds() / self.tracks.len() as i64);
            print!(" (avg. {} per track)", utils::formatted_duration(average));
        };
        println!("\n");
    }

    /// Print playlist with the given formatting style
    pub fn print_playlist(&self, style: &FormattingStyle) {
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
            if has_valid_file_extension {
                value
            } else {
                // Can't use `with_extension` here since it will replace anything after the last dot,
                // which will alter the name if it contains a date separated by dots for example.
                utils::append_extension_to_pathbuf(value, "csv")
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
            }
            log::info!("Overwriting existing file");
        }

        self.write_playlist_file(path.as_path())
    }

    fn read_serato_txt(
        path: &Path,
        name: String,
        header: &BTreeMap<String, usize>,
        data: &[BTreeMap<String, String>],
    ) -> Result<Playlist> {
        let required_fields = ["artist", "name"];
        for field in required_fields {
            if !header.contains_key(field) {
                anyhow::bail!("Serato TXT missing required field: '{}'", field)
            }
        }

        let (playlist_name, playlist_date) = Self::parse_serato_playlist_info(&data[0]);
        let tracks = Self::parse_serato_tracks_from_data(data, playlist_date);
        let total_duration = utils::get_total_playtime(&tracks);
        let max_artist_length: usize = tracks.iter().map(|t| t.artist_length()).max().unwrap_or(0);
        let max_title_length: usize = tracks.iter().map(|t| t.title_length()).max().unwrap_or(0);
        let max_playtime_length: usize = Self::get_max_playtime_length(&tracks);

        Ok(Playlist {
            date: playlist_date,
            file: PathBuf::from(path),
            file_format: FileFormat::Txt,
            name: if playlist_name.is_empty() {
                name
            } else {
                playlist_name
            },
            playlist_type: PlaylistType::Serato,
            tracks,
            max_artist_length,
            max_title_length,
            max_playtime_length,
            total_duration,
        })
    }

    fn read_rekordbox_txt(
        path: &Path,
        name: String,
        header: &BTreeMap<String, usize>,
        data: &[BTreeMap<String, String>],
    ) -> Result<Playlist> {
        let required_fields = ["Artist", "Track Title"];
        for field in required_fields {
            if !header.contains_key(field) {
                anyhow::bail!("Rekordbox TXT missing required field: '{}'", field)
            }
        }

        let mut tracks: Vec<Track> = {
            data.iter()
                .map(|row| {
                    Track::new(
                        row.get(required_fields[0]).unwrap().to_string(),
                        row.get(required_fields[1]).unwrap().to_string(),
                    )
                })
                .collect()
        };

        // Remove consecutive duplicates
        tracks.dedup();

        let total_duration = utils::get_total_playtime(&tracks);

        let max_artist_length: usize = tracks.iter().map(|t| t.artist_length()).max().unwrap_or(0);
        let max_title_length: usize = tracks.iter().map(|t| t.title_length()).max().unwrap_or(0);
        // rekordbox does not have any start or play time info :(
        let max_playtime_length: usize = 0;

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

    /// Print a simple playlist without any formatting
    fn print_simple_playlist(&self) {
        for track in &self.tracks {
            println!("{track}");
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

        let header = if self.max_playtime_length > 0 {
            format!(
                "{:<index_width$}   {:<artist_width$}   {:<title_width$}   {:>playtime_width$}",
                "#",
                "ARTIST",
                "TITLE",
                "PLAYTIME",
                index_width = index_width,
                artist_width = self.max_artist_length,
                title_width = self.max_title_length,
                playtime_width = max(
                    self.max_playtime_length,
                    "PLAYTIME".to_string().chars().count()
                )
            )
        } else {
            format!(
                "{:<index_width$}   {:<artist_width$}   {:<title_width$}",
                "#",
                "ARTIST",
                "TITLE",
                index_width = index_width,
                artist_width = self.max_artist_length,
                title_width = self.max_title_length,
            )
        };

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
                    utils::formatted_duration(d).green()
                } else {
                    "".normal()
                },
                index_width = index_width,
                artist_width = self.max_artist_length,
                title_width = self.max_title_length,
                playtime_width = if self.max_playtime_length > 0 {
                    max(
                        self.max_playtime_length,
                        "PLAYTIME".to_string().chars().count(),
                    )
                } else {
                    0
                }
            );
        }

        println!("{divider}");
    }

    /// Get playlist format enum from file extension
    fn playlist_format(file: &Path) -> Result<FileFormat> {
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
            Some(ext) => ext.to_str().unwrap(),
        };
        FileFormat::from_str(extension)
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
        for track in &self.tracks {
            writer.write_record([track.artist.clone(), "-".to_string(), track.title.clone()])?;
        }
        writer.flush()?;
        Ok(())
    }

    /// Write tracks to TXT file
    fn write_txt_file(&self, filepath: &Path) -> Result<()> {
        let mut file = File::create(filepath)?;
        for track in &self.tracks {
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

    fn get_max_playtime_length(tracks: &[Track]) -> usize {
        tracks
            .iter()
            .map(|t| {
                utils::formatted_duration(t.play_time.unwrap_or(Duration::seconds(0)))
                    .chars()
                    .count()
            })
            .max()
            .unwrap_or(0)
    }

    // Split txt content to lines, and each line to separate items
    fn read_txt_lines(dest: &mut str) -> Vec<Vec<String>> {
        let lines: Vec<Vec<String>> = {
            let initial_lines: Vec<Vec<String>> = dest
                .lines()
                .map(|s| s.split('\t').map(|l| l.trim().to_string()).collect())
                .collect();

            // Serato txt has a divider on the second line
            if initial_lines[1][0].chars().all(|c| c == '-') {
                // This is a Serato txt, need to do some extra parsing here...
                let header_line: String = initial_lines[0][0].clone();
                let header_fields: Vec<String> = header_line
                    .replace("     ", "\t")
                    .split('\t')
                    .filter_map(|s| {
                        let v = s.trim();
                        if s.is_empty() {
                            None
                        } else {
                            Some(v.to_string())
                        }
                    })
                    .collect();
                let mut field_indices: Vec<usize> = header_fields
                    .iter()
                    .map(|field| header_line.find(field).unwrap())
                    .collect();
                field_indices.reverse();
                let mut serato_lines: Vec<Vec<String>> = vec![];
                for line in initial_lines {
                    if line[0].chars().all(|c| c == '-') {
                        continue;
                    }
                    let mut split_line: Vec<String> = vec![];
                    let mut remainder: String = line[0].to_string();
                    for index in &field_indices {
                        if *index >= remainder.len() {
                            continue;
                        }
                        let (string_left, value) = remainder.split_at(*index);
                        split_line.push(value.trim().to_string());
                        remainder = string_left.to_string();
                    }
                    split_line.reverse();
                    while split_line.len() < header_fields.len() {
                        split_line.push(String::new());
                    }
                    serato_lines.push(split_line);
                }
                serato_lines
            } else {
                initial_lines
            }
        };
        lines
    }

    /// Parse Serato track data
    fn parse_serato_tracks_from_data(
        data: &[BTreeMap<String, String>],
        playlist_date: Option<NaiveDateTime>,
    ) -> Vec<Track> {
        let start_date = playlist_date.unwrap_or_default().date();
        let initial_tracks: Vec<Track> = {
            data[1..]
                .iter()
                .map(|row| {
                    let start_time: Option<NaiveDateTime> = {
                        match row.get("start time") {
                            None => None,
                            Some(t) => match NaiveTime::parse_from_str(t, "%H.%M.%S %Z") {
                                Ok(n) => Some(NaiveDateTime::new(start_date, n)),
                                Err(_) => None,
                            },
                        }
                    };
                    let end_time: Option<NaiveDateTime> = {
                        match row.get("end time") {
                            None => None,
                            Some(t) => match NaiveTime::parse_from_str(t, "%H.%M.%S %Z") {
                                Ok(n) => Some(NaiveDateTime::new(start_date, n)),
                                Err(_) => None,
                            },
                        }
                    };
                    let play_time: Option<Duration> = {
                        match row.get("playtime") {
                            None => None,
                            Some(t) => match NaiveTime::parse_from_str(t, "%H:%M:%S") {
                                Ok(n) => Some(
                                    Duration::hours(i64::from(n.hour()))
                                        + Duration::minutes(i64::from(n.minute()))
                                        + Duration::seconds(i64::from(n.second())),
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

        // Remove consecutive duplicates
        let mut new_tracks_index: usize = 0;
        let mut tracks: Vec<Track> = vec![initial_tracks[0].clone()];
        for track in initial_tracks[1..].iter() {
            let previous_track = &tracks[new_tracks_index];
            if *previous_track == *track {
                // duplicate track -> add playtime to previous and skip
                tracks[new_tracks_index] = previous_track.clone() + track.play_time
            } else {
                // new track, append to playlist
                tracks.push(track.clone());
                new_tracks_index += 1;
            }
        }
        tracks
    }
}
