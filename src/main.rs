#![warn(clippy::cargo)]

mod playlist;
mod track;
mod utils;

#[cfg(test)]
mod playlist_tests;

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::Local;
use clap::Parser;
use log::LevelFilter;

use crate::playlist::Playlist;
use crate::utils::{CliConfig, FormattingStyle, Level};

/// Command line arguments
///
/// Basic info is read from `Cargo.toml`
/// See Clap `Derive` documentation for details:
/// <https://docs.rs/clap/latest/clap/_derive/index.html>
#[derive(Parser)]
#[command(
    author,
    version,
    about = "DJ playlist formatting utility.",
    long_about = "DJ playlist formatting utility. Reads raw playlist files and creates a nicely formatted version.",
    arg_required_else_help = true
)]
struct Args {
    /// Playlist file to process (required)
    file: String,

    /// Optional output path to save playlist to
    output: Option<String>,

    /// Overwrite an existing output file
    #[arg(short, long, help = "Use default save dir")]
    default: bool,

    /// Overwrite an existing output file
    #[arg(short, long, help = "Overwrite an existing file")]
    force: bool,

    /// Log level
    #[arg(value_enum, short, long, help = "Log level", value_name = "LEVEL")]
    log: Option<Level>,

    /// Basic formatting style
    #[arg(short, long, help = "Use basic print formatting style", conflicts_with = "numbered")]
    basic: bool,

    /// Numbered formatting style
    #[arg(short, long, help = "Use numbered print formatting style", conflicts_with = "basic")]
    numbered: bool,

    /// Numbered formatting style
    #[arg(short, long, help = "Don't print playlist")]
    quiet: bool,

    /// Write playlist to file
    #[arg(
        short,
        long,
        help = "Save formatted playlist to file",
        long_help = "Save formatted playlist to file. This can be a name or path. Empty value will use default path",
        value_name = "OUTPUT_FILE",
        conflicts_with = "output"
    )]
    save: Option<Option<String>>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    init_logger(&args.log);
    let absolute_input_path = parse_input_path(&args.file)?;
    let config = CliConfig::from_args(args);
    let playlist = Playlist::new(&absolute_input_path)?;

    if config.style == FormattingStyle::Pretty {
        playlist.print_info();
    }
    if !config.quiet {
        playlist.print_tracks(&config.style);
    }
    if config.save {
        playlist.save_to_file(config.output_path, config.force, config.default)?
    }

    Ok(())
}

fn init_logger(log_level: &Option<Level>) {
    // Get logging level to use
    let log_level_filter = match log_level {
        None => LevelFilter::Info,
        Some(ref level) => level.to_log_filter(),
    };
    // Init logger with timestamps
    env_logger::Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}]: {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, log_level_filter)
        .init();

    log::debug!("Using log level: {}", log_level_filter);
}

fn parse_input_path(input: &str) -> Result<PathBuf> {
    let input_file = input.trim();
    if input_file.is_empty() {
        anyhow::bail!("Empty input file");
    }
    let filepath = Path::new(input_file);
    if !filepath.is_file() {
        anyhow::bail!(
            "File does not exist or is not accessible: '{}'",
            dunce::simplified(filepath).display()
        );
    }
    let absolute_input_path = dunce::canonicalize(filepath)?;
    log::info!("Playlist file: {}", absolute_input_path.display());
    Ok(absolute_input_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_config_basic() {
        // Simulating "--basic"
        let args = Args {
            file: "playlist.txt".into(),
            output: None,
            default: false,
            force: false,
            log: None,
            basic: true,
            numbered: false,
            quiet: false,
            save: None,
        };
        let config = CliConfig::from_args(args);
        assert_eq!(config.style, FormattingStyle::Basic);
        assert_eq!(config.output_path, None);
    }

    #[test]
    fn cli_config_with_output_and_force() {
        // Simulating "--output some/path --force"
        let args = Args {
            file: "playlist.txt".into(),
            output: Some("some/path/playlist-2024".into()),
            default: false,
            force: true,
            log: None,
            basic: false,
            numbered: false,
            quiet: false,
            save: None,
        };

        let config = CliConfig::from_args(args);

        assert!(config.force);
        assert!(config.save);
        assert_eq!(config.output_path, Some("some/path/playlist-2024".into()));
    }

    #[test]
    fn cli_config_with_save() {
        // Simulating "--save"
        let args = Args {
            file: "playlist.txt".into(),
            output: None,
            default: false,
            force: false,
            log: None,
            basic: false,
            numbered: false,
            quiet: false,
            save: Some(None),
        };

        let config = CliConfig::from_args(args);

        assert!(config.save);
        assert_eq!(config.output_path, None);
    }

    #[test]
    fn cli_config_with_save_with_path() {
        // Simulating "--save playlist1.csv"
        let args = Args {
            file: "playlist.txt".into(),
            output: None,
            default: false,
            force: false,
            log: None,
            basic: false,
            numbered: false,
            quiet: false,
            save: Some(Some("playlist1.csv".to_string())),
        };

        let config = CliConfig::from_args(args);

        assert!(config.save);
        assert_eq!(config.output_path, Some("playlist1.csv".to_string()));
    }
}
