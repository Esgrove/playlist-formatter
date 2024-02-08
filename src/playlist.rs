use crate::track::Track;
use crate::utils;
use crate::utils::{FileFormat, FormattingStyle, PlaylistType};

use anyhow::{anyhow, Context, Result};
use chrono::{Duration, NaiveDateTime, NaiveTime, Timelike};
use colored::Colorize;
use csv::Reader;
use encoding_rs_io::DecodeReaderBytes;
use home::home_dir;
use strum::IntoEnumIterator;

use std::cmp::max;
use std::collections::BTreeMap;
use std::ffi::OsStr;
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
    // helpers for formatting
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
            FormattingStyle::Simple => self.print_simple_playlist(),
            FormattingStyle::Numbered => self.print_numbered_playlist(),
            FormattingStyle::Pretty => self.print_pretty_playlist(),
        }
    }

    /// Get output file path.
    pub fn get_output_file_path(&self, filepath: Option<String>) -> PathBuf {
        let potential_path: Option<PathBuf> = filepath
            .map(|value| value.trim().to_string())
            .filter(|trimmed| !trimmed.is_empty())
            .map(PathBuf::from);

        potential_path.map_or_else(
            || {
                // If `potential_path` is `None`, use the default save directory.
                let mut save_dir = self.default_save_dir();
                log::info!("Using default save dir: {}", save_dir.display());
                save_dir.push(self.file.with_extension("csv"));
                save_dir
            },
            |value| {
                log::info!("Using output path: {}", value.display());
                match value
                    .extension()
                    .and_then(OsStr::to_str)
                    .map_or(false, |ext| FileFormat::from_str(ext).is_ok())
                {
                    true => value,
                    false => utils::append_extension_to_path(value, "csv"),
                }
            },
        )
    }

    /// Write playlist to given file.
    pub fn save_playlist_to_file(&self, filepath: Option<String>, overwrite_existing: bool) -> Result<()> {
        let path = self.get_output_file_path(filepath);
        log::info!("Saving to: {}", path.display());
        if path.is_file() {
            log::info!("Output file already exists: {}", path.display());
            if !overwrite_existing {
                anyhow::bail!("use the {} option overwrite an existing output file", "force".bold());
            }
            log::info!("Overwriting existing file");
        }

        let extension = path
            .extension()
            .ok_or_else(|| anyhow!("Output file has no extension"))?
            .to_str()
            .ok_or_else(|| anyhow!("Output file extension cannot be converted to string"))?
            .to_lowercase();

        match extension.as_str() {
            "csv" => self.write_csv_file(&path),
            "txt" => self.write_txt_file(&path),
            _ => anyhow::bail!("Unsupported file extension"),
        }
    }

    /// Read a .txt playlist file.
    /// This can be either a Rekordbox or Serato exported playlist.
    fn read_txt(path: &Path) -> Result<Playlist> {
        let file = File::open(path)?;
        // Rekordbox encodes txt files in UTF-16 :(
        // This implementation is far from ideal since it reads everything into a single string,
        // but this way can easily convert to utf-8 using encoding_rs_io
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
        log::debug!("txt headers ({}): {:?}", header_map.keys().len(), header_map.keys());

        // Map track data to a dictionary (header key: track value)
        let data: Vec<BTreeMap<String, String>> = {
            lines[1..]
                .iter()
                .map(|line| {
                    let mut items: BTreeMap<String, String> = BTreeMap::new();
                    // header map contains the index of the value corresponding to the key
                    for (key, index) in &header_map {
                        let value = &line[*index];
                        items.insert(key.to_string(), value.to_string());
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
            .ok_or_else(|| anyhow!("File name not found after dropping extension"))?
            .to_str()
            .ok_or_else(|| anyhow!("File name contains invalid Unicode"))?
            .to_string();

        // Check playlist type
        if header_map.contains_key("name") {
            log::debug!("Detected Serato TXT");
            Playlist::read_serato_txt(path, name, &header_map, &data)
        } else if header_map.contains_key("#") {
            // Rekordbox txt: first line contains headers, line starts with '#'.
            log::debug!("Detected Rekordbox TXT");
            Playlist::read_rekordbox_txt(path, name, &header_map, &data)
        } else {
            anyhow::bail!("Input file does not seem to be a valid Serato or Rekordbox txt playlist");
        }
    }

    /// Read a .csv playlist file.
    /// This can be either a Serato playlist or an already formatted file.
    fn read_csv(path: &Path) -> Result<Playlist> {
        let mut reader =
            Reader::from_path(path).with_context(|| format!("Failed to open CSV file: '{}'", path.display()))?;

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

        log::debug!("CSV headers ({}): {:?}", header_map.keys().len(), header_map.keys());

        let data = Self::map_track_data(&mut reader, &header_map);
        log::debug!("Rows ({}):", data.len());
        for row in &data {
            log::debug!("{:?}", row);
        }

        let formatted_fields = ["artist", "title"];
        let formatted_csv: bool = formatted_fields.into_iter().all(|field| header_map.contains_key(field));
        if formatted_csv {
            // this is an already-formatted CSV
            Self::read_formatted_csv(path, data)
        } else {
            // this should be a Serato CSV
            let required_serato_fields = ["name", "artist"];
            for field in required_serato_fields {
                if !header_map.contains_key(field) {
                    anyhow::bail!("Serato CSV missing required field: '{}'", field)
                }
            }
            Self::read_serato_csv(path, data)
        }
    }

    /// Read a formatted CSV playlist file.
    fn read_formatted_csv(path: &Path, data: Vec<BTreeMap<String, String>>) -> Result<Playlist> {
        // TODO: fix data reading
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
            playlist_type: PlaylistType::Formatted,
            tracks,
            max_artist_length,
            max_title_length,
            max_playtime_length,
            total_duration,
        })
    }

    /// Read a Serato CSV playlist file.
    fn read_serato_csv(path: &Path, data: Vec<BTreeMap<String, String>>) -> Result<Playlist> {
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

    /// Map track data to a dictionary (header key: track value).
    fn map_track_data(
        reader: &mut Reader<File>,
        header_map: &BTreeMap<String, usize>,
    ) -> Vec<BTreeMap<String, String>> {
        reader
            .records()
            .filter_map(|s| s.ok())
            .map(|record| {
                let mut items: BTreeMap<String, String> = BTreeMap::new();
                for (name, index) in header_map {
                    let value = &record[*index];
                    items.insert(name.to_string(), value.to_string());
                }
                items
            })
            .collect()
    }

    /// Parse first row data from a Serato playlist.
    ///
    /// This row should contain the playlist name and start datetime.
    fn parse_serato_playlist_info(data: &BTreeMap<String, String>) -> (String, Option<NaiveDateTime>) {
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

    /// Read data from a Serato txt playlist.
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
        let name = if playlist_name.is_empty() { name } else { playlist_name };
        let tracks = Self::parse_serato_tracks_from_data(data, playlist_date);
        let total_duration = utils::get_total_playtime(&tracks);
        let max_artist_length: usize = tracks.iter().map(|t| t.artist_length()).max().unwrap_or(0);
        let max_title_length: usize = tracks.iter().map(|t| t.title_length()).max().unwrap_or(0);
        let max_playtime_length: usize = Self::get_max_playtime_length(&tracks);

        Ok(Playlist {
            date: playlist_date,
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

    /// Read data from a Rekordbox txt playlist.
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

        // Rekordbox does not have any start or play time info :(
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

        let max_artist_length: usize = tracks.iter().map(|t| t.artist_length()).max().unwrap_or(0);
        let max_title_length: usize = tracks.iter().map(|t| t.title_length()).max().unwrap_or(0);

        Ok(Playlist {
            date: None,
            file: PathBuf::from(path),
            file_format: FileFormat::Txt,
            name,
            playlist_type: PlaylistType::Rekordbox,
            tracks,
            max_artist_length,
            max_title_length,
            max_playtime_length: 0,
            total_duration: None,
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
            println!("{:>0index_width$}: {}", index + 1, track, index_width = index_width);
        }
    }

    /// Print a nicely formatted playlist
    fn print_pretty_playlist(&self) {
        let index_width = self.tracks.len().to_string().chars().count();
        let playtime_width = if self.max_playtime_length > 0 {
            max(self.max_playtime_length, "PLAYTIME".to_string().chars().count())
        } else {
            0
        };

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
                playtime_width = playtime_width
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
            let playtime = if let Some(d) = track.play_time {
                utils::formatted_duration(d).green()
            } else {
                "".normal()
            };
            println!(
                "{:>0index_width$}   {:<artist_width$}   {:<title_width$}   {:>playtime_width$}",
                index + 1,
                track.artist,
                track.title,
                playtime,
                index_width = index_width,
                artist_width = self.max_artist_length,
                title_width = self.max_title_length,
                playtime_width = playtime_width
            );
        }

        println!("{divider}");
    }

    /// Return default save directory for playlist output file.
    ///
    /// This will first try to use the Dropbox playlist directory if it exists on disk.
    /// After that, it will try the get the directory of the input file.
    /// Otherwise, returns an empty path so the file will go to the current working directory.
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

    /// Write tracks to CSV file
    fn write_csv_file(&self, filepath: &Path) -> Result<()> {
        let mut writer = csv::Writer::from_path(filepath)?;
        writer.write_record(["artist", "", "title", "playtime", "start time", "end time"])?;
        for track in &self.tracks {
            let duration = match track.play_time {
                None => String::new(),
                Some(d) => utils::formatted_duration(d),
            };
            let start_time = match track.start_time {
                None => String::new(),
                Some(t) => t.format("%Y.%m.%d %H:%M:%S").to_string(),
            };
            let end_time = match track.end_time {
                None => String::new(),
                Some(t) => t.format("%Y.%m.%d %H:%M:%S").to_string(),
            };
            writer.write_record([
                track.artist.clone(),
                "-".to_string(),
                track.title.clone(),
                duration,
                start_time,
                end_time,
            ])?;
        }
        // Add total duration
        if let Some(t) = self.total_duration {
            writer.write_record([
                String::new(),
                String::new(),
                String::new(),
                utils::formatted_duration(t),
                String::new(),
                String::new(),
            ])?;
        }
        writer.flush()?;
        Ok(())
    }

    /// Write tracks to TXT file
    fn write_txt_file(&self, filepath: &Path) -> Result<()> {
        let mut file = File::create(filepath)?;
        for track in &self.tracks {
            file.write_all(format!("{}\n", track).as_ref())?;
        }
        Ok(())
    }

    /// Get DJ playlist directory path in Dropbox if it exists
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

    /// Get playlist format enum from file extension.
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
            Some(ext) => ext.to_str().context("Failed to parse file extension")?,
        };
        FileFormat::from_str(extension)
    }

    /// Get the longest formatted track playtime length in number of chars.
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

    /// Split txt content string to lines, and each line to separate items
    fn read_txt_lines(dest: &mut str) -> Vec<Vec<String>> {
        // Convert to lines and split each line from tab. This handles Rekordbox data.
        let initial_lines: Vec<Vec<String>> = dest
            .lines()
            .map(|s| s.split('\t').map(|l| l.trim().to_string()).collect())
            .collect();

        // Check if this is a Serato txt: Serato has a divider on the second line
        if initial_lines[1][0].chars().all(|c| c == '-') {
            // This is a Serato txt, need to do some extra parsing here...
            let header_line: String = initial_lines[0][0].clone();
            let column_names: Vec<String> = header_line
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
            // Get starting location of each column item on a line
            let mut column_start_indices: Vec<usize> = column_names
                .iter()
                .map(|field| header_line.find(field).unwrap())
                .collect();
            // Extract each column item from the line, starting from the end of the line
            column_start_indices.reverse();
            let mut serato_lines: Vec<Vec<String>> = vec![];
            for line in initial_lines {
                // skip the divider lines
                if line[0].chars().all(|c| c == '-') {
                    continue;
                }
                let mut split_line: Vec<String> = vec![];
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
        } else {
            initial_lines
        }
    }

    /// Parse Serato track data from dictionary
    fn parse_serato_tracks_from_data(
        data: &[BTreeMap<String, String>],
        playlist_date: Option<NaiveDateTime>,
    ) -> Vec<Track> {
        let start_date = playlist_date.unwrap_or_default().date();
        let initial_tracks: Vec<Track> = {
            data[1..]
                .iter()
                .map(|row| {
                    let start_time: Option<NaiveDateTime> = row
                        .get("start time")
                        .and_then(|t| NaiveTime::parse_from_str(t, "%H.%M.%S %Z").ok())
                        .map(|n| NaiveDateTime::new(start_date, n));

                    let end_time: Option<NaiveDateTime> = row
                        .get("end time")
                        .and_then(|t| NaiveTime::parse_from_str(t, "%H.%M.%S %Z").ok())
                        .map(|n| NaiveDateTime::new(start_date, n));

                    let play_time = match row.get("playtime") {
                        Some(t) => NaiveTime::parse_from_str(t, "%H:%M:%S").ok().map(|n| {
                            Duration::hours(i64::from(n.hour()))
                                + Duration::minutes(i64::from(n.minute()))
                                + Duration::seconds(i64::from(n.second()))
                        }),
                        None => start_time.and_then(|start| end_time.map(|end| end - start)),
                    };
                    Track::new_with_time(
                        row.get("artist").unwrap_or(&"".to_string()).to_string(),
                        row.get("name").unwrap_or(&"".to_string()).to_string(),
                        start_time,
                        end_time,
                        play_time,
                    )
                })
                .collect()
        };

        // Remove consecutive duplicates
        let mut index: usize = 0;
        let mut tracks: Vec<Track> = vec![initial_tracks[0].clone()];
        for track in initial_tracks[1..].iter() {
            let previous_track = &tracks[index];
            if *previous_track == *track {
                // duplicate track -> add playtime to previous and skip
                tracks[index] += track.play_time;
                tracks[index].end_time = track.end_time;
            } else {
                // new track, append to playlist
                tracks.push(track.clone());
                index += 1;
            }
        }
        tracks
    }
}
