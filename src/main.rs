mod formatter;
use anyhow::Result;
use chrono::Local;
use clap::Parser;
use formatter::{FormattingStyle, Playlist};
use log::LevelFilter;
use std::io::Write;
use std::path::Path;

#[derive(clap::ValueEnum, Clone, Debug)]
enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

/// Command line arguments
///
/// Basic info is read from `Cargo.toml`
/// See Clap `Derive` documentation for details:
/// https://docs.rs/clap/latest/clap/_derive/index.html
#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = "DJ playlist formatting utility. Reads raw playlist files and creates a nicely formatted version.",
    arg_required_else_help = true
)]
struct Args {
    /// Overwrite an existing file
    #[arg(short, long, help = "Overwrite an existing file")]
    overwrite: bool,

    /// Log level
    #[arg(value_enum, short, long, help = "Log level", value_name = "LEVEL")]
    log: Option<Level>,

    #[arg(
        short,
        long,
        help = "Use simple print formatting style",
        conflicts_with = "numbered"
    )]
    simple: bool,

    #[arg(
        short,
        long,
        help = "Use numbered print formatting style",
        conflicts_with = "simple"
    )]
    numbered: bool,

    /// Playlist file to process (required)
    file: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let log_level_filter = match args.log {
        None => LevelFilter::Info,
        Some(level) => match level {
            Level::Debug => LevelFilter::Debug,
            Level::Info => LevelFilter::Info,
            Level::Warn => LevelFilter::Warn,
            Level::Error => LevelFilter::Error,
        },
    };

    // init logger with timestamps
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

    log::debug!("Using log level: {:?}", log_level_filter);

    let filepath = Path::new(&args.file);
    if !filepath.is_file() {
        anyhow::bail!(
            "file does not exist or is not accessible: '{}'",
            filepath.display()
        );
    }

    log::debug!("Playlist file: {}", filepath.display());

    let style = if args.simple {
        FormattingStyle::Simple
    } else if args.numbered {
        FormattingStyle::Numbered
    } else {
        FormattingStyle::Pretty
    };

    let formatter = Playlist::new(filepath);

    log::debug!("{:#?}", formatter);

    if style == FormattingStyle::Pretty {
        formatter.print_info();
    }

    formatter.print_playlist(style);

    Ok(())
}
