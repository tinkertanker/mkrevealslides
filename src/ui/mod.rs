use crate::errors::ArgumentError;
use crate::slide::io::{find_slides, SlideFile};
use crate::ui::cli::{CliArgs, Commands};
use crate::ui::conf::PresentationConfigFile;
use anyhow::Error;
use std::path::PathBuf;
use std::{env, fs};
use tracing::{trace, warn};

pub mod cli;
pub mod conf;

/// The logical representation of a presentation configuration
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PresentationConfig {
    /// Title of the presentation
    pub title: String,
    /// Absolute path to the final output file
    /// # Note
    /// The file does not need to exist, but the directories do!
    pub output_file: PathBuf,
    /// Absolute path to the template file
    pub template_file: PathBuf,
    /// Absolute paths to files to include in the presentation
    pub include_files: Vec<SlideFile>,
}

impl PresentationConfig {
    /// Attempts to validate this PresentationConfig
    /// In particular, it checks that any paths
    /// specified are valid, and those that need to be
    /// accessed can be accessed.
    fn validate(&self) -> Result<(), Error> {
        trace!("Validating PresentationConfig");
        trace!("Checking output_file");
        if !self.output_file.is_absolute() {
            return Err(anyhow::Error::from(ArgumentError::new(
                "output_file".to_string(),
                self.output_file.to_str().unwrap_or("<invalid path>"),
                "Output file must be an absolute path".to_string(),
            )));
        }

        // does it exist and is it a directory? if so, reject it
        if self.output_file.is_dir() {
            return Err(Error::from(ArgumentError::new(
                "output_file".to_string(),
                self.output_file.to_str().unwrap_or("<invalid path>"),
                "Output file cannot be a directory".to_string(),
            )));
        }

        // does it exist and is it a file?
        if self.output_file.is_file() {
            // if it exists, we will warn about overwriting it
            warn!(
                "Output file at `{}` already exists, will overwrite",
                self.output_file.display()
            );
        } else {
            // Let's have a look at it's parent directory and check if it at least exists
            if let Some(output_dir) = self.output_file.parent() {
                if output_dir.is_dir() {
                    // it exists, so we can continue
                } else {
                    return Err(Error::from(ArgumentError::new(
                        "output_file".to_string(),
                        self.output_file.to_str().unwrap_or("<invalid path>"),
                        "Output file's parent directory does not exist".to_string(),
                    )));
                }
            } else {
                // the parent directory is root (e.g. / or C:\), or is something that doesn't really make sense
                // e.g. /a/b/c/.. (the parent of this is actually /a/b)
                // todo: prefixes should be handled
                return Err(Error::from(ArgumentError::new(
                    "output_file".to_string(),
                    self.output_file.to_str().unwrap_or("<invalid path>"),
                    "The directory that will contain this output file is invalid".to_string(),
                )));
            }
        }
        trace!("Checking template_file");
        if !self.template_file.is_absolute() {
            return Err(Error::from(ArgumentError::new(
                "template_file".to_string(),
                self.template_file.to_str().unwrap_or("<invalid path>"),
                "Template file must be an absolute path".to_string(),
            )));
        }

        if !self.template_file.is_file() {
            return Err(Error::from(ArgumentError::new(
                "template_file".to_string(),
                self.template_file.to_str().unwrap_or("<invalid path>"),
                "Template file does not exist or cannot be read".to_string(),
            )));
        }
        trace!("Checking include_files");
        for include_file in &self.include_files {
            trace!("Checking include_file: {:?}", include_file);
            include_file.validate()?;
        }
        Ok(())
    }
}

/// Attempts to convert CLI user input to PresentationConfig
/// All paths will be converted to absolute paths with respect to the current working directory.
/// (i.e. the directory the command was executed in)
impl TryFrom<CliArgs> for PresentationConfig {
    type Error = anyhow::Error;

    fn try_from(args: CliArgs) -> Result<Self, Self::Error> {
        match args.command {
            Commands::FromConfig { config_path } => {
                let config = PresentationConfigFile::read_config_file(config_path)?;
                Ok(Self::try_from(config)?)
            }
            Commands::FromCli {
                title,
                slide_dir,
                template_file,
                output_dir,
                output_file,
            } => {
                trace!("Converting CLI args to PresentationConfig");
                let cwd = fs::canonicalize(env::current_dir()?)?;
                let slide_title = if let Some(title) = title {
                    title
                } else {
                    "Untitled Presentation".to_string()
                };
                let slides = find_slides(&cwd.join(slide_dir))?;
                let cfg = PresentationConfig {
                    title: slide_title,
                    output_file: cwd.join(output_dir).join(output_file),
                    template_file: cwd.join(template_file),
                    include_files: slides,
                };
                cfg.validate()?;
                Ok(cfg)
            }
        }
    }
}

/// Attempts to convert a PresentationConfigFile to PresentationConfig
/// Validates and converts relative paths to absolute paths in the process
impl TryFrom<PresentationConfigFile> for PresentationConfig {
    type Error = anyhow::Error;

    fn try_from(config: PresentationConfigFile) -> Result<Self, Self::Error> {
        trace!("Attempting to convert PresentationConfigFile to PresentationConfig");
        let include_files_abs_paths = config
            .include_files
            .iter()
            .map(|relative_pth| {
                config
                    .working_directory
                    .join(&config.slide_dir)
                    .join(relative_pth)
            })
            .collect::<Vec<PathBuf>>();
        trace!(
            "Converted {} include_file paths to abs paths",
            include_files_abs_paths.len()
        );
        let include_files = if include_files_abs_paths.is_empty() {
            // let's try to search for slides
            find_slides(&config.working_directory.join(config.slide_dir))?
        } else {
            let mut sf = include_files_abs_paths
                .iter()
                .map(|fp| SlideFile::try_from(fp.clone()))
                .collect::<Result<Vec<SlideFile>, anyhow::Error>>()?;
            sf.sort();
            sf
        };

        let cfg = PresentationConfig {
            title: config.title,
            output_file: config.working_directory.join(config.output_file),
            template_file: config.working_directory.join(config.template_file),
            include_files,
        };
        cfg.validate()?;
        Ok(cfg)
    }
}
