#![warn(clippy::cargo)]

mod playlist;
mod track;
mod utils;

use crate::playlist::Playlist;
use crate::utils::{FormattingStyle, Level};

use anyhow::Result;
use chrono::Local;
use clap::Parser;

use std::io::Write;
use std::path::Path;

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

/// Run playlist formatting based on command line arguments
fn run_playlist_formatter_cli(args: Args) -> Result<()> {
    let input_file = args.file.trim();
    if input_file.is_empty() {
        anyhow::bail!("empty input file");
    }
    let filepath = Path::new(input_file);
    if !filepath.is_file() {
        anyhow::bail!("file does not exist or is not accessible: '{}'", filepath.display());
    }
    log::debug!("Playlist file: {}", filepath.display());

    // formatting style to use
    let style = if args.basic {
        FormattingStyle::Basic
    } else if args.numbered {
        FormattingStyle::Numbered
    } else {
        FormattingStyle::Pretty
    };
    log::debug!("Formatting style: {style}");

    let formatter = Playlist::new(filepath)?;
    log::trace!("{:#?}", formatter);

    if style == FormattingStyle::Pretty {
        formatter.print_info();
    }

    formatter.print_playlist(&style);

    if let Some(path) = args.save {
        formatter.save_playlist_to_file(path, args.force)?;
    } else if args.output.is_some() {
        formatter.save_playlist_to_file(args.output, args.force)?;
    }

    Ok(())
}

fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Get logging level to use
    let log_level_filter = match args.log {
        None => log::LevelFilter::Info,
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

    run_playlist_formatter_cli(args)
}
