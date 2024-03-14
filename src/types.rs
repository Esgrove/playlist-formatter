use anyhow::anyhow;
use strum_macros::{Display, EnumIter};

use std::str::FromStr;

use crate::Args;

/// Playlist file type
#[derive(Debug, Clone, PartialEq, EnumIter, Display)]
pub enum FileFormat {
    Txt,
    Csv,
}

/// Export file type
#[derive(Debug, Clone, PartialEq, EnumIter, Display)]
pub enum OutputFormat {
    Txt,
    Csv,
    Xlsx,
}

/// Output formatting style for playlist printing
#[derive(Default, Debug, Clone, PartialEq, Display)]
pub enum FormattingStyle {
    /// Basic formatting for sharing playlist text online
    Basic,
    /// Basic formatting but with track numbers
    Numbered,
    /// Pretty formatting for human readable formatted CLI output
    #[default]
    Pretty,
}

/// Which DJ software is the playlist from.
///
/// Each software has its own formatting style.
/// `Formatted` means it was already processed by this program.
#[derive(Debug, Clone, PartialEq, Display)]
pub enum PlaylistType {
    Rekordbox,
    Serato,
    Formatted,
}

/// Logging level
#[derive(clap::ValueEnum, Clone, Debug, Display)]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Default, Debug, Clone)]
pub struct CliConfig {
    pub default: bool,
    pub force: bool,
    pub quiet: bool,
    pub save: bool,
    pub output_path: Option<String>,
    pub style: FormattingStyle,
}

impl CliConfig {
    /// Create config from command line args.
    pub fn from_args(args: Args) -> Self {
        let style = if args.basic {
            FormattingStyle::Basic
        } else if args.numbered {
            FormattingStyle::Numbered
        } else {
            FormattingStyle::Pretty
        };
        log::debug!("Formatting style: {style}");

        let (save, output_path) = if args.save.is_some() {
            log::debug!("Save option specified");
            (true, args.save.unwrap())
        } else if args.output.is_some() {
            log::debug!("Output path specified");
            (true, args.output)
        } else {
            (false, None)
        };

        CliConfig {
            force: args.force,
            default: args.default,
            quiet: args.quiet,
            save,
            output_path,
            style,
        }
    }
}

impl Level {
    pub fn to_log_filter(&self) -> log::LevelFilter {
        match self {
            Level::Trace => log::LevelFilter::Trace,
            Level::Debug => log::LevelFilter::Debug,
            Level::Info => log::LevelFilter::Info,
            Level::Warn => log::LevelFilter::Warn,
            Level::Error => log::LevelFilter::Error,
        }
    }
}

/// Convert string to `FileFormat` enum
impl FromStr for FileFormat {
    type Err = anyhow::Error;
    fn from_str(input: &str) -> anyhow::Result<FileFormat> {
        match input.to_lowercase().trim() {
            "csv" => Ok(FileFormat::Csv),
            "txt" => Ok(FileFormat::Txt),
            "" => Err(anyhow!("Can't convert empty string to file format")),
            _ => Err(anyhow!("Unsupported file format: '{input}'")),
        }
    }
}

/// Convert string to `OutputFormat` enum
impl FromStr for OutputFormat {
    type Err = anyhow::Error;
    fn from_str(input: &str) -> anyhow::Result<OutputFormat> {
        match input.to_lowercase().trim() {
            "csv" => Ok(OutputFormat::Csv),
            "txt" => Ok(OutputFormat::Txt),
            "xlsx" => Ok(OutputFormat::Xlsx),
            "" => Err(anyhow!("Can't convert empty string to file format")),
            _ => Err(anyhow!("Unsupported file format: '{input}'")),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{FileFormat, OutputFormat};
    use std::str::FromStr;

    #[test]
    fn file_format_valid_format() {
        assert_eq!(FileFormat::from_str("csv").unwrap(), FileFormat::Csv);
        assert_eq!(FileFormat::from_str("CSV").unwrap(), FileFormat::Csv);
        assert_eq!(FileFormat::from_str("txt").unwrap(), FileFormat::Txt);
        assert_eq!(FileFormat::from_str("TXT").unwrap(), FileFormat::Txt);
    }

    #[test]
    fn file_format_unsupported_format() {
        let result = FileFormat::from_str("mp3");
        assert!(
            result.is_err(),
            "Expected an error for unsupported format, but got {:?}",
            result
        );
        if let Err(e) = result {
            assert_eq!(e.to_string(), "Unsupported file format: 'mp3'");
        }
    }

    #[test]
    fn file_format_empty_string() {
        let result = FileFormat::from_str("");
        assert!(
            result.is_err(),
            "Expected an error for empty string, but got {:?}",
            result
        );
        if let Err(e) = result {
            assert_eq!(e.to_string(), "Can't convert empty string to file format");
        }
    }

    #[test]
    fn output_format() {
        assert_eq!(OutputFormat::from_str("csv").unwrap(), OutputFormat::Csv);
        assert_eq!(OutputFormat::from_str("CSV").unwrap(), OutputFormat::Csv);
        assert_eq!(OutputFormat::from_str("txt").unwrap(), OutputFormat::Txt);
        assert_eq!(OutputFormat::from_str("TXT").unwrap(), OutputFormat::Txt);
        assert_eq!(OutputFormat::from_str("xlsx").unwrap(), OutputFormat::Xlsx);
        assert_eq!(OutputFormat::from_str("XLSX").unwrap(), OutputFormat::Xlsx);
    }
}
