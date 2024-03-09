use std::cmp::max;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::string::String;

use anyhow::{anyhow, Context, Result};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, Timelike};
use colored::Colorize;
use csv::Reader;
use encoding_rs_io::DecodeReaderBytes;
use home::home_dir;
use lazy_static::lazy_static;
use regex::Regex;
use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, RowNum, Workbook};
use strum::IntoEnumIterator;

use crate::track::Track;
use crate::utils;
use crate::utils::{FileFormat, FormattingStyle, OutputFormat, PlaylistType};

lazy_static! {
    static ref RE_DD_MM_YYYY: Regex =
        Regex::new(r"(\d{1,2})\.(\d{1,2})\.(\d{4})").expect("Failed to create regex pattern for dd.mm.yyyy");
    static ref RE_YYYY_MM_DD: Regex =
        Regex::new(r"(\d{4})\.(\d{1,2})\.(\d{1,2})").expect("Failed to create regex pattern for yyyy.mm.dd");
}

/// Holds imported playlist data
#[derive(Debug)]
pub struct Playlist {
    pub date: Option<NaiveDateTime>,
    pub file_format: FileFormat,
    pub file: PathBuf,
    pub name: String,
    pub playlist_type: PlaylistType,
    pub total_duration: Option<TimeDelta>,
    tracks: Vec<Track>,
    // helpers for formatting
    max_artist_length: usize,
    max_title_length: usize,
    max_playtime_length: usize,
}

impl Playlist {
    /// Initialize playlist from given filepath
    pub fn new(file: &Path) -> Result<Playlist> {
        match Self::playlist_format(file)? {
            FileFormat::Csv => Self::read_csv(file),
            FileFormat::Txt => Self::read_txt(file),
        }
    }

    /// Print playlist information (but not the tracks themselves)
    pub fn print_info(&self) {
        println!("Playlist: {}", self.name.green());
        println!("Filepath: {}", self.file.display());
        println!(
            "Format: {}, Type: {}, Date: {}",
            self.file_format.to_string().cyan(),
            self.playlist_type.to_string().cyan(),
            if let Some(date) = self.date {
                date.format("%Y.%m.%d %H:%M").to_string().magenta()
            } else {
                "unknown".to_string().yellow()
            }
        );
        print!("Tracks: {}", self.tracks.len());
        if let Some(duration) = self.total_duration {
            print!(", Total duration: {}", utils::formatted_duration(duration));
            let average = TimeDelta::try_seconds(duration.num_seconds() / self.tracks.len() as i64).unwrap();
            print!(" (avg. {} per track)", utils::formatted_duration(average));
        };
        println!("\n");
    }

    /// Print playlist with the given formatting style.
    pub fn print_playlist(&self, style: &FormattingStyle) {
        match style {
            FormattingStyle::Basic => self.print_simple_playlist(),
            FormattingStyle::Numbered => self.print_numbered_playlist(),
            FormattingStyle::Pretty => self.print_pretty_playlist(),
        }
    }

    /// Print a simple playlist without any formatting.
    fn print_simple_playlist(&self) {
        for track in &self.tracks {
            println!("{track}");
        }
    }

    /// Print a simple playlist with track numbers.
    fn print_numbered_playlist(&self) {
        let index_width = self.tracks.len().to_string().chars().count();
        for (index, track) in self.tracks.iter().enumerate() {
            println!("{:>0index_width$}: {}", index + 1, track, index_width = index_width);
        }
    }

    /// Print a nicely formatted playlist.
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

    /// Get output file path.
    pub fn get_output_file_path(&self, filepath: Option<String>) -> PathBuf {
        let potential_path: Option<PathBuf> = filepath
            .map(|value| value.trim().to_string())
            .filter(|trimmed| !trimmed.is_empty())
            .map(PathBuf::from);

        potential_path.map_or_else(
            || {
                // If `potential_path` is `None`, use the default save directory.
                let save_dir = self.default_save_dir();
                log::info!("Using default save dir: {}", save_dir.display());
                save_dir.join(&self.file)
            },
            |value| match value
                .extension()
                .and_then(OsStr::to_str)
                .map_or(false, |ext| OutputFormat::from_str(ext).is_ok())
            {
                true => value,
                false => utils::append_extension_to_path(value, "csv"),
            },
        )
    }

    /// Write playlist to given file.
    pub fn save_playlist_to_file(&self, filepath: Option<String>, overwrite_existing: bool) -> Result<()> {
        let path = self.get_output_file_path(filepath);
        log::info!("Saving to: {}", path.display());
        if path.is_file() {
            if !overwrite_existing {
                log::error!("Output file already exists: {}", path.display());
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
            "xlsx" => self.write_excel_file(&path),
            _ => anyhow::bail!("Unsupported file extension"),
        }
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
        // Add total TimeDelta
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

    /// Write tracks to Excel file
    fn write_excel_file(&self, filepath: &Path) -> Result<()> {
        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet().set_name(self.name.clone())?;

        let header_format = Format::new()
            .set_bold()
            .set_border_bottom(FormatBorder::Medium)
            .set_background_color("C6E0B4");

        // Write header
        sheet.write_string_with_format(0, 0, "Artist", &header_format)?;
        sheet.write_string_with_format(0, 1, "", &header_format)?;
        sheet.write_string_with_format(0, 2, "Title", &header_format)?;
        sheet.write_string_with_format(0, 3, "Playtime", &header_format)?;
        sheet.write_string_with_format(0, 4, "Start Time", &header_format)?;
        sheet.write_string_with_format(0, 5, "End Time", &header_format)?;

        let duration_format = Format::new().set_align(FormatAlign::Right).set_num_format("h:mm:ss");

        // Write tracks
        for (i, track) in self.tracks.iter().enumerate() {
            let row = (i + 1) as RowNum;
            let duration = track.play_time.map_or(String::new(), utils::formatted_duration);
            let start_time = track
                .start_time
                .map_or(String::new(), |t| t.format("%Y.%m.%d %H:%M:%S").to_string());
            let end_time = track
                .end_time
                .map_or(String::new(), |t| t.format("%Y.%m.%d %H:%M:%S").to_string());

            sheet.write_string(row, 0, &track.artist)?;
            sheet.write_string(row, 1, "-")?;
            sheet.write_string(row, 2, &track.title)?;
            sheet.write_string_with_format(row, 3, &duration, &duration_format)?;
            sheet.write_string(row, 4, &start_time)?;
            sheet.write_string(row, 5, &end_time)?;
        }

        // Add total TimeDelta at the end
        if let Some(t) = self.total_duration {
            let total_row = (self.tracks.len() + 1) as RowNum;
            let formatted_duration = utils::formatted_duration(t);
            sheet.write_string_with_format(total_row, 3, &formatted_duration, &duration_format)?;
        }

        sheet.autofit();

        workbook.save(filepath)?;
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
        log::trace!("Lines ({}):", lines.len());
        log::trace!("{:#?}", lines);

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
        log::trace!("txt headers ({}): {:?}", header_map.keys().len(), header_map.keys());

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

        log::trace!("Rows ({}):", data.len());
        for row in &data {
            log::trace!("{:#?}", row);
        }

        // Drop the file extension from file name
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

        log::trace!("CSV headers ({}): {:?}", header_map.keys().len(), header_map.keys());

        let data = Self::map_track_data(&mut reader, &header_map);
        log::trace!("Rows ({}):", data.len());
        for row in &data {
            log::trace!("{:?}", row);
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
        let mut playlist_date = match data.get("start time") {
            None => None,
            Some(time) => NaiveDateTime::parse_from_str(time, "%d.%m.%Y, %H.%M.%S %Z").ok(),
        };
        if playlist_date.is_none() && !playlist_name.is_empty() {
            playlist_date = Self::extract_datetime_from_name(&playlist_name);
        }
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
        let date = if playlist_date.is_none() {
            Self::extract_datetime_from_name(&name)
        } else {
            playlist_date
        };
        let tracks = Self::parse_serato_tracks_from_data(data, playlist_date);
        let total_duration = utils::get_total_playtime(&tracks);
        let max_artist_length: usize = tracks.iter().map(|t| t.artist_length()).max().unwrap_or(0);
        let max_title_length: usize = tracks.iter().map(|t| t.title_length()).max().unwrap_or(0);
        let max_playtime_length: usize = Self::get_max_playtime_length(&tracks);

        Ok(Playlist {
            date,
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

        let date = Self::extract_datetime_from_name(&name);

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
            date,
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

    /// Get playlist format enum from the file extension.
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
                utils::formatted_duration(t.play_time.unwrap_or(TimeDelta::try_seconds(0).unwrap()))
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
                        Some(t) => NaiveTime::parse_from_str(t, "%H:%M:%S").ok().and_then(|n| {
                            let hours = TimeDelta::try_hours(i64::from(n.hour()))?;
                            let minutes = TimeDelta::try_minutes(i64::from(n.minute()))?;
                            let seconds = TimeDelta::try_seconds(i64::from(n.second()))?;
                            Some(hours + minutes + seconds)
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

    fn extract_datetime_from_name(input: &str) -> Option<NaiveDateTime> {
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
}
