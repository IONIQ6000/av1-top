// Command-line interface for the daemon
// Uses clap for parsing arguments

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "av1d",
    about = "AV1 Daemon - Automated AV1 transcoding with Intel QSV",
    version,
    author
)]
pub struct Args {
    /// Directory to watch for media files
    #[arg(short, long, value_name = "DIR")]
    pub directory: Option<PathBuf>,
    
    /// Path to configuration file
    #[arg(short, long, value_name = "FILE", default_value = "~/.config/av1janitor/config.toml")]
    pub config: String,
    
    /// Run once and exit (don't loop continuously)
    #[arg(long)]
    pub once: bool,
    
    /// Dry run mode (analyze but don't transcode)
    #[arg(long)]
    pub dry_run: bool,
    
    /// Number of concurrent transcodes (default: 1)
    #[arg(long, value_name = "N", default_value = "1")]
    pub concurrent: usize,
    
    /// Verbosity level (can be repeated: -v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

impl Args {
    /// Get the config file path, expanding ~ if needed
    pub fn get_config_path(&self) -> PathBuf {
        let path = self.config.replace('~', &std::env::var("HOME").unwrap_or_default());
        PathBuf::from(path)
    }
}

