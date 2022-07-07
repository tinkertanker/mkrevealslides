use crate::error_handling::AppError;
use crate::io::{find_slides, grab_file_names_from_path_bufs};
use crate::val::validate_file_path;
use clap::ArgMatches;
use serde::Deserialize;
use std::{env, fs};
use std::path::PathBuf;
use tracing::trace;

#[derive(Debug, Deserialize, Clone)]
#[non_exhaustive]
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
    #[serde(default)]
    pub include_files: Vec<PathBuf>,

    #[serde(skip)]
    pub config_path: PathBuf,
}

pub type ArgumentError = (String, String);

impl PresentationConfig {
    /// Creates a new PresentationConfig
    /// Should only be used for testing purposes
    fn new(title: String, slide_dir: PathBuf, output_file: PathBuf,
           template_file: PathBuf, include_files: Vec<PathBuf>,
    config_path: PathBuf) -> Self {
        Self {
            title,
            slide_dir,
            output_file,
            template_file,
            include_files,
            config_path
        }
    }

    pub fn read_config_file(config_file_path: PathBuf) -> Result<Self, AppError> {
        trace!(
            "Attempting to read config file: {}",
            config_file_path.display()
        );
        let config_str = fs::read_to_string(&config_file_path)?;
        trace!("Config file read: {} bytes", config_str.len());
        let config_parent_dir = &config_file_path.parent().ok_or_else(|| {
            AppError::new(
            "Could not find parent directory of config file"
            )
        })?;

        let mut config: Self = serde_yaml::from_str(&config_str)?;
        if config.include_files.is_empty() {
            let search_path = &config_parent_dir.join(&config.slide_dir);
            trace!("Searching for slides in: {}", search_path.display());
            config.include_files = Self::search_for_slides(search_path)?;
        }
        config.config_path = config_file_path;
        config.convert_paths_to_full_paths()?;
        config.validate_include_paths()?;
        Ok(config)
    }
    /// Processes arguments provided to the program
    /// and builds the logical configuration from that
    ///
    /// # Errors
    /// Returns error if the arguments are invalid (e.g. missing required arguments)
    /// Returns error if it could not parse the file indices into a vector of FileEntry structs
    /// Returns error if it could not read the template file
    /// Returns error if it cannot access the current working directory
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
        let include_files = Self::search_for_slides(&slide_dir)?;

        let mut cfg = Self {
            title,
            slide_dir,
            output_file,
            template_file,
            include_files,
            config_path: env::current_dir()?,
        };
        cfg.convert_paths_to_full_paths()?;
        cfg.validate_include_paths()?;
        Ok(cfg)
    }

    /// Searches for slides in the slide dir
    /// and returns their slide file names
    fn search_for_slides(slide_dir: &PathBuf) -> Result<Vec<PathBuf>, AppError> {
        let slides = find_slides(slide_dir)?;
        let slide_file_names = grab_file_names_from_path_bufs(&slides)?;
        Ok(slide_file_names)
    }

    /// Validates the include_files field checking if files actually exist.
    /// Should only be ran when a config file is read, after all the paths
    /// have been converted to full paths
    fn validate_include_paths(&self) -> Result<(), AppError> {
        for include_file in self.include_files.iter() {
            trace!(
                "Validating include file: {:?}",
                include_file
            );
            validate_file_path(
                include_file
                    .to_str()
                    .ok_or_else(||
                        AppError::new("Could not convert file path to string"))?,
            )?;
        }

        Ok(())
    }

    /// Converts all the paths in the config to full paths,
    /// relative to `config_path`
    fn convert_paths_to_full_paths(&mut self) -> Result<(), AppError> {
        if !self.config_path.is_file() {
            return Err(AppError::new("Config path is not a file"));
        }
        let config_dir = self.config_path.parent().unwrap();
        self.slide_dir = config_dir.join(&self.slide_dir);
        self.output_file = config_dir.join(&self.output_file);
        self.template_file = config_dir.join(&self.template_file);
        for include_file in self.include_files.iter_mut() {
            *include_file = self.slide_dir.join(&include_file);
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use super::*;
    use tempfile::tempdir;

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
        fs::create_dir(&tmp_dir.path().join("slides")).unwrap();
        fs::write(&cfg_path, cfg_str).unwrap();
        let cfg = PresentationConfig::read_config_file(cfg_path).unwrap();
        assert_eq!(cfg.title, "Test Presentation");
        assert_eq!(cfg.slide_dir, tmp_dir.path().join("slides"));
        assert_eq!(cfg.output_file, tmp_dir.path().join("output.html"));
        assert_eq!(cfg.template_file, tmp_dir.path().join("template.html"));

    }

    #[test]
    fn test_convert_paths_to_full_paths() {
        let tmp_dir = tempdir().unwrap();
        let cfg_path = tmp_dir.path().join("config.yaml");
        File::create(&cfg_path).unwrap();
        let mut cfg = PresentationConfig::new(
            "Test Presentation".to_string(),
            PathBuf::from("slides"),
            PathBuf::from("output.html"),
            PathBuf::from("template.html"),
            vec![
                PathBuf::from("slide_1.md"),
                PathBuf::from("slide_2.md"),
                PathBuf::from("slide_3.md")
            ],
            cfg_path,
        );
        cfg.convert_paths_to_full_paths().expect("Could not convert paths to full paths");
        assert_eq!(
            cfg.slide_dir,
            tmp_dir.path().join("slides")
        );
        assert_eq!(
            cfg.output_file,
            tmp_dir.path().join("output.html")
        );
        assert_eq!(
            cfg.template_file,
            tmp_dir.path().join("template.html")
        );
        assert_eq!(cfg.include_files, vec![
            tmp_dir.path().join("slides/slide_1.md"),
            tmp_dir.path().join("slides/slide_2.md"),
            tmp_dir.path().join("slides/slide_3.md")
        ]);

    }
}