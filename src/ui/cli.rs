use clap::{Parser, Subcommand};
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use tracing::Level;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct CliArgs {
    /// Increase the level of tracing/logging.
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: usize,

    #[clap(subcommand)]
    pub command: Commands,
}

/// Subcommands available to the CLI interface
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Creates your presentation from a config file
    FromConfig {
        /// Path to your config file
        #[clap(parse(from_os_str))]
        config_path: PathBuf,
    },
    /// Creates your presentation from CLI arguments
    FromCli {
        /// Title of the presentation to make
        #[clap(short, long)]
        title: Option<String>,

        /// Directory to search for slides in
        #[clap(parse(try_from_os_str=validate_slide_dir))]
        slide_dir: PathBuf,

        /// Path to the template file to use
        #[clap(parse(try_from_os_str=file_exists))]
        template_file: PathBuf,

        /// Output directory to place generated slides in
        #[clap(parse(from_os_str))]
        output_dir: PathBuf,

        /// Output filename to use
        #[clap(parse(from_os_str), default_value = "index.html")]
        output_file: PathBuf,
    },
}

/// Checks if the given path is
/// - a directory
/// - exists
/// - can be read
fn dir_exists(s: &OsStr) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);
    if path.is_dir() {
        match fs::read_dir(&path) {
            Ok(_) => Ok(path),
            Err(e) => Err(format!(
                "Could not read directory `{}`: {}",
                path.display(),
                e
            )),
        }
    } else if !path.exists() {
        Err(format!("`{}` does not exist", path.display()))
    } else {
        Err(format!("`{}` is not a directory", path.display()))
    }
}

/// Just checks if the slide dir provided is
/// - a directory
/// - exists
/// - can be read
/// - contains files/directories
fn validate_slide_dir(slide_dir: &OsStr) -> Result<PathBuf, String> {
    let path_to_dir = dir_exists(slide_dir)?;

    match path_to_dir.read_dir() {
        Ok(mut dir) => {
            if dir.next().is_none() {
                return Err(format!("`{}` is empty", path_to_dir.display()));
            }
            for dir_entry in dir {
                if let Err(e) = dir_entry {
                    return Err(e.to_string());
                }
            }
            Ok(path_to_dir)
        }
        Err(e) => {
            return Err(format!(
                "Could not read directory `{}`: {}",
                path_to_dir.display(),
                e
            ))
        }
    }
}

/// Checks if the given path is:
/// - a file
/// - exists
/// - can be read
fn file_exists(s: &OsStr) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);
    if path.is_file() {
        match File::open(&path) {
            Ok(_) => Ok(path),
            Err(e) => Err(format!("Could not open file: `{}`", e)),
        }
    } else if !path.exists() {
        Err(format!("`{}` does not exist", path.display()))
    } else {
        Err(format!("`{}` is not a file", path.display()))
    }
}

impl CliArgs {
    /// Returns an appropriate log level based on the verbosity level configured
    pub fn get_log_level(&self) -> Level {
        match self.verbose {
            0 => Level::ERROR,
            1 => Level::WARN,
            2 => Level::INFO,
            3 => Level::DEBUG,
            _ => Level::TRACE,
        }
    }
}
