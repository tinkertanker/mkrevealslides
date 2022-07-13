use anyhow::Context;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use tracing::trace;

/// A PresentationConfigFile which has been deserialized
#[derive(Debug, Deserialize)]
pub struct PresentationConfigFile {
    pub title: String,
    /// Slide directory relative to the directory of the config file
    pub slide_dir: PathBuf,
    /// Output file relative to the directory of the config file
    pub output_file: PathBuf,
    /// Template file relative to the directory of the config file
    pub template_file: PathBuf,

    /// Include files relative to the directory of the config file
    #[serde(default)]
    pub include_files: Vec<PathBuf>,
    #[serde(skip)]
    /// Absolute path of the directory containing the config file
    pub working_directory: PathBuf,
}

impl PresentationConfigFile {
    /// Reads a YAML configuration file from the config file path
    ///
    /// # Arguments
    /// * `config_file_path` - The path to the configuration file
    ///
    /// # Returns
    /// A PresentationConfigFile if the file is valid
    ///
    /// # Errors
    /// - If the file is not valid YAML
    /// - If the parent directory of the file cannot be accessed
    pub fn read_config_file(config_file_path: PathBuf) -> Result<Self, anyhow::Error> {
        trace!(
            "Attempting to read config file: {}",
            config_file_path.display()
        );
        let config_str = fs::read_to_string(&config_file_path)?;
        trace!("Config file read: {} bytes", config_str.len());
        let config_parent_dir = &config_file_path
            .parent()
            .with_context(|| "Could not find parent directory of config file")?;

        let mut config: Self = serde_yaml::from_str(&config_str)?;

        let p_dir = fs::canonicalize(config_parent_dir)?;
        config.working_directory = p_dir;
        Ok(config)
    }
}

#[cfg(test)]
mod test {

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
        let cfg = PresentationConfigFile::read_config_file(cfg_path).unwrap();
        assert_eq!(cfg.title, "Test Presentation");
        assert_eq!(cfg.slide_dir, PathBuf::from("slides"));
        assert_eq!(cfg.output_file, PathBuf::from("output.html"));
        assert_eq!(cfg.template_file, PathBuf::from("template.html"));
        assert_eq!(cfg.working_directory, tmp_dir.path());
    }
}
