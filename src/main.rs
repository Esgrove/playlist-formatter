mod cli;

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Parser;
use log::LevelFilter;

use crate::cli::{Args, CliConfig, FormattingStyle, Level};

use playlist_formatter::playlist::Playlist;

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
        match &config.style {
            FormattingStyle::Basic => playlist.print_simple_playlist(),
            FormattingStyle::Numbered => playlist.print_numbered_playlist(),
            FormattingStyle::Pretty => playlist.print_pretty_playlist(),
        }
    }
    if config.save {
        playlist.save_to_file(config.output_path, config.force, config.default, &config.output_format)?
    }

    Ok(())
}

fn init_logger(log_level: &Option<Level>) {
    // Get logging level to use
    let log_level_filter = match log_level {
        None => LevelFilter::Info,
        Some(level) => level.to_log_filter(),
    };
    // Init logger with timestamps
    env_logger::Builder::new()
        .format(|formatter, record| match record.level() {
            log::Level::Info => {
                writeln!(formatter, "{}", record.args())
            }
            _ => {
                writeln!(formatter, "[{}]: {}", record.level(), record.args())
            }
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
