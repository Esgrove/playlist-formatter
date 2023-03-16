use clap::{Parser};

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
    arg_required_else_help = true
)]
struct Args {
    /// Overwrite an existing file
    #[arg(short, long, help = "Overwrite an existing file")]
    overwrite: bool,

    /// Playlist file to process (required)
    file: String,
}

fn main() {
    let args = Args::parse();
    println!("file: {:?}", args.file);
}
