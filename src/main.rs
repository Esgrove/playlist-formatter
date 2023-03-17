mod formatter;

use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Parser;

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

    /// Playlist file to process (required)
    file: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let filepath = PathBuf::from(&args.file);
    if !filepath.is_file() {
        anyhow::bail!(
            "file does not exist or is not accessible: '{}'",
            filepath.display()
        );
    }

    println!("File: {}", filepath.display());

    let mut formatter = formatter::Playlist::new(filepath);

    Ok(())
}
