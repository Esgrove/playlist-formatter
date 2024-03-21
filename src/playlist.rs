use std::cmp::max;
use std::collections::BTreeMap;
use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::string::String;

use anyhow::{anyhow, Context, Result};
use chrono::{NaiveDateTime, TimeDelta};
use colored::Colorize;
use csv::Reader;
use encoding_rs_io::DecodeReaderBytes;
use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, RowNum, Workbook};

use super::track::Track;
use super::types::{FileFormat, OutputFormat, PlaylistType};
use super::{formatted, rekordbox, serato, utils};

/// Holds imported playlist data
#[derive(Debug)]
pub struct Playlist {
    pub date: Option<NaiveDateTime>,
    pub file_format: FileFormat,
    pub file: PathBuf,
    pub name: String,
    pub playlist_type: PlaylistType,
    pub total_duration: Option<TimeDelta>,
    pub tracks: Vec<Track>,
    // helpers for formatting
    pub max_artist_length: usize,
    pub max_title_length: usize,
    pub max_playtime_length: usize,
}

impl Playlist {
    /// Initialize playlist from given filepath
    pub fn new(file: &Path) -> Result<Playlist> {
        match utils::playlist_format(file)? {
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

    /// Print a simple playlist without any formatting.
    pub fn print_simple_playlist(&self) {
        for track in &self.tracks {
            println!("{track}");
        }
    }

    /// Print a simple playlist with track numbers.
    pub fn print_numbered_playlist(&self) {
        let index_width = self.tracks.len().to_string().chars().count();
        for (index, track) in self.tracks.iter().enumerate() {
            println!("{:>0index_width$}: {}", index + 1, track, index_width = index_width);
        }
    }

    /// Print a nicely formatted playlist.
    pub fn print_pretty_playlist(&self) {
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
    pub fn get_output_file_path(&self, filepath: Option<String>, use_default_dir: bool) -> PathBuf {
        let default_save_dir = self.default_save_dir();

        let potential_path: Option<PathBuf> = filepath
            .map(|value| value.trim().to_string())
            .filter(|trimmed| !trimmed.is_empty())
            .map(PathBuf::from);

        let output_path = if let Some(path) = potential_path {
            let absolute_path = dunce::canonicalize(&path).unwrap_or(path);
            if use_default_dir {
                absolute_path
                    .file_name()
                    .map(|filename| default_save_dir.join(filename))
                    .unwrap_or(absolute_path)
            } else {
                absolute_path
            }
        } else {
            // If `potential_path` is `None`, use the default save directory.
            log::debug!("Using default save path: {}", default_save_dir.display());
            default_save_dir.join(&self.file)
        };

        match output_path
            .extension()
            .and_then(OsStr::to_str)
            .map_or(false, |ext| OutputFormat::from_str(ext).is_ok())
        {
            true => output_path,
            false => utils::append_extension_to_path(output_path, "csv"),
        }
    }

    /// Write playlist to given file.
    pub fn save_to_file(
        &self,
        filepath: Option<String>,
        overwrite_existing: bool,
        use_default_dir: bool,
    ) -> Result<()> {
        let path = self.get_output_file_path(filepath, use_default_dir);
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
        utils::dropbox_save_dir().unwrap_or_else(|| {
            dunce::canonicalize(&self.file)
                .map(|path| {
                    path.parent().map_or_else(
                        || env::current_dir().unwrap_or_else(|_| PathBuf::new()),
                        |parent| parent.to_path_buf(),
                    )
                })
                .unwrap_or_else(|error| {
                    log::error!("Failed to resolve full path to input file: {}", error);
                    env::current_dir().unwrap_or_else(|_| PathBuf::new())
                })
        })
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
            serato::read_serato_txt(path, name, &header_map, &data)
        } else if header_map.contains_key("#") {
            // Rekordbox txt: first line contains headers, line starts with '#'.
            log::debug!("Detected Rekordbox TXT");
            rekordbox::read_rekordbox_txt(path, name, &header_map, &data)
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

        // Check if this is an already-formatted CSV
        let formatted_fields = ["Artist", "", "Song"];
        if formatted_fields.into_iter().all(|field| header_map.contains_key(field)) {
            formatted::read_formatted_csv(path, data)
        } else {
            // This should be a Serato CSV
            let required_serato_fields = ["name", "artist"];
            for field in required_serato_fields {
                if !header_map.contains_key(field) {
                    anyhow::bail!("Serato CSV missing required field: '{}'", field)
                }
            }
            serato::read_serato_csv(path, data)
        }
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

    /// Split txt content string to lines, and each line to separate items
    fn read_txt_lines(text: &mut str) -> Vec<Vec<String>> {
        // Convert to lines and split each line from tab. This handles Rekordbox data.
        let initial_lines: Vec<Vec<String>> = text
            .lines()
            .map(|s| s.split('\t').map(|l| l.trim().to_string()).collect())
            .collect();

        // Check if this is a Serato txt: Serato has a divider on the second line
        if initial_lines[1][0].chars().all(|c| c == '-') {
            // This is a Serato txt, need to do some extra parsing here...
            serato::read_serato_txt_lines(initial_lines)
        } else {
            initial_lines
        }
    }
}
