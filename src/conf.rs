use crate::error_handling::AppError;
use crate::io::{find_included_slides, grab_file_names_from_path_bufs};
use crate::val::validate_file_path;
use clap::ArgMatches;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use tracing::trace;

#[derive(Debug, Deserialize, Clone)]
pub struct PresentationConfig {
    /// Title of the presentation
    pub title: String,
    /// Path to the directory containing the slides
    pub slide_dir: PathBuf,
    /// Path to the final output file.
    /// # Note
    /// The file does not need to exist, but the directories do!
    pub output_file: PathBuf,
    /// Path to the template file to use for the presentation.
    pub template_file: PathBuf,
    /// Paths to the files to include in the presentation.
    ///
    /// # Note
    /// relative to slide_dir!
    pub include_files: Option<Vec<String>>,
}

pub type ArgumentError = (String, String);

impl PresentationConfig {
    pub fn read_config_file(config_file_path: &PathBuf) -> Result<Self, AppError> {
        trace!(
            "Attempting to read config file: {}",
            config_file_path.display()
        );
        let config_str = fs::read_to_string(config_file_path)?;
        trace!("Config file read: {} bytes", config_str.len());
        let config: Self = serde_yaml::from_str(&config_str)?;
        Ok(config)
    }
    /// Processes arguments provided to the program
    /// and builds the logical configuration from that
    ///
    /// # Errors
    /// Returns error if the arguments are invalid (e.g. missing required arguments)
    /// Returns error if it could not parse the file indices into a vector of FileEntry structs
    /// Returns error if it could not read the template file
    pub fn process_args(args: ArgMatches) -> Result<Self, AppError> {
        let slide_dir = args
            .get_one::<PathBuf>("slide_dir")
            .ok_or_else(|| {
                (
                    "slide_dir".to_string(),
                    "Slide directory is required".to_string(),
                )
            })?
            .clone();
        let output_file = args
            .get_one::<PathBuf>("output_file")
            .ok_or_else(|| {
                (
                    "output_file".to_string(),
                    "Output file is required".to_string(),
                )
            })?
            .clone();
        let template_file = args
            .get_one::<PathBuf>("template_file")
            .ok_or_else(|| {
                (
                    "template_file".to_string(),
                    "Template file is required".to_string(),
                )
            })?
            .clone();
        let title = args
            .get_one::<String>("title")
            .ok_or_else(|| ("title".to_string(), "Title is required".to_string()))?
            .clone();

        let include_files = find_included_slides(&slide_dir)?;
        let include_files = grab_file_names_from_path_bufs(&include_files)?;

        Ok(Self {
            title,
            slide_dir,
            output_file,
            template_file,
            include_files: Some(include_files),
        })
    }

    /// Validates the include_files field checking if files actually exist.
    /// The validation only runs if the include_files field is None,
    /// otherwise, the program will automatically search for files.
    pub fn validate_include_paths(&self) -> Result<(), AppError> {
        if let Some(include_files) = &self.include_files {
            for include_file in include_files {
                let file_path = self.slide_dir.join(include_file);
                trace!(
                    "Validating include file: {} at {}",
                    include_file,
                    file_path.display()
                );
                validate_file_path(
                    file_path
                        .to_str()
                        .ok_or_else(|| AppError::new("Could not convert file path to string"))?,
                )?;
            }
        }
        Ok(())
    }

    /// Converts the include_files and slide_dir fields into a vector of PathBufs
    pub fn to_full_paths(&self) -> Result<Vec<PathBuf>, AppError> {
        if let Some(include_files) = &self.include_files {
            let mut full_paths = Vec::new();
            for include_file in include_files {
                let file_path = self.slide_dir.join(include_file);
                full_paths.push(file_path);
            }
            Ok(full_paths)
        } else {
            Ok(find_included_slides(&self.slide_dir)?)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tempfile::tempdir;

    macro_rules! pb {
        ($path:expr) => {
            PathBuf::from($path)
        };
    }

    #[test]
    fn test_read_config_file() {
        let tmp_dir = tempdir().unwrap();
        let cfg_path = tmp_dir.path().join("config.yaml");
        let cfg_str = r#"
title: "Test Presentation"
slide_dir: "slides"
output_file: "output.html"
template_file: "template.html"
        "#;
        fs::write(&cfg_path, cfg_str).unwrap();
        let cfg = PresentationConfig::read_config_file(&cfg_path).unwrap();
        assert_eq!(cfg.title, "Test Presentation");
        assert_eq!(cfg.slide_dir, pb!("slides"));
        assert_eq!(cfg.output_file, pb!("output.html"));
        assert_eq!(cfg.template_file, pb!("template.html"));

    }
}