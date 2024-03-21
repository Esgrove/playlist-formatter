use clap::Parser;
use strum_macros::Display;

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
pub struct Args {
    /// Playlist file to process (required)
    pub file: String,

    /// Log level
    #[arg(value_enum, short, long, help = "Log level", value_name = "LEVEL")]
    pub log: Option<Level>,

    /// Optional output path to save playlist to
    output: Option<String>,

    /// Overwrite an existing output file
    #[arg(short, long, help = "Use default save dir")]
    default: bool,

    /// Overwrite an existing output file
    #[arg(short, long, help = "Overwrite an existing file")]
    force: bool,

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

/// Logging level
#[derive(clap::ValueEnum, Clone, Debug, Display)]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
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
