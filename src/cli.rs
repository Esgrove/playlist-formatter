use clap::Parser;
use strum_macros::Display;

use playlist_formatter::types::OutputFormat;

/// DJ playlist formatting utility
#[derive(Parser)]
#[allow(clippy::struct_excessive_bools)]
#[command(
    author,
    version,
    about,
    long_about = "DJ playlist formatting utility. Reads raw playlist files and creates a nicely formatted version.",
    arg_required_else_help = true
)]
pub struct Args {
    /// Playlist file to process
    pub file: String,

    /// Optional output path to save playlist to
    output: Option<String>,

    /// Log level
    #[arg(value_enum, short, long, value_name = "LEVEL")]
    pub log: Option<Level>,

    /// Output format
    #[arg(value_enum, short = 't', long = "type", value_name = "OUTPUT_FORMAT")]
    pub output_format: Option<OutputFormat>,

    /// Use default save directory
    #[arg(short, long)]
    default: bool,

    /// Overwrite an existing output file
    #[arg(short, long)]
    force: bool,

    /// Use basic print formatting style
    #[arg(short, long, conflicts_with = "numbered")]
    basic: bool,

    /// Use numbered print formatting style
    #[arg(short, long, conflicts_with = "basic")]
    numbered: bool,

    /// Don't print playlist
    #[arg(short, long)]
    quiet: bool,

    /// Save formatted playlist to file
    #[arg(
        short,
        long,
        long_help = "Save formatted playlist to file. This can be a name or path. Empty value will use default path",
        value_name = "OUTPUT_FILE",
        conflicts_with = "output"
    )]
    #[allow(clippy::option_option)]
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
#[derive(Default, Debug, Clone, PartialEq, Eq, Display)]
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
#[allow(clippy::struct_excessive_bools)]
pub struct CliConfig {
    pub default: bool,
    pub force: bool,
    pub quiet: bool,
    pub save: bool,
    pub style: FormattingStyle,
    pub output_path: Option<String>,
    pub output_format: OutputFormat,
}

impl CliConfig {
    /// Create config from command line args.
    pub fn from_args(args: Args) -> Self {
        let style = if args.basic {
            FormattingStyle::Basic
        } else if args.numbered {
            FormattingStyle::Numbered
        } else {
            FormattingStyle::default()
        };
        log::debug!("Formatting style: {style}");

        let (save, output_path) = if let Some(save_path) = args.save {
            log::debug!("Save option specified");
            (true, save_path)
        } else if args.output.is_some() {
            log::debug!("Output path specified");
            (true, args.output)
        } else {
            (false, None)
        };

        Self {
            force: args.force,
            default: args.default,
            quiet: args.quiet,
            save,
            style,
            output_path,
            output_format: args.output_format.unwrap_or_default(),
        }
    }
}

impl Level {
    pub const fn to_log_filter(&self) -> log::LevelFilter {
        match self {
            Self::Trace => log::LevelFilter::Trace,
            Self::Debug => log::LevelFilter::Debug,
            Self::Info => log::LevelFilter::Info,
            Self::Warn => log::LevelFilter::Warn,
            Self::Error => log::LevelFilter::Error,
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
            output_format: None,
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
            output_format: None,
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
            output_format: None,
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
            output_format: None,
        };

        let config = CliConfig::from_args(args);

        assert!(config.save);
        assert_eq!(config.output_path, Some("playlist1.csv".to_string()));
    }
}
