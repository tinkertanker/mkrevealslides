use std::path::PathBuf;
use clap::ArgMatches;
use serde::{Deserialize};
use crate::{build_proc_pq, fetch_file_indices, indices_and_paths_to_entries, just_file_paths};
use crate::error_handling::AppError;
use crate::val::validate_file_path;

#[derive(Debug, Deserialize)]
pub struct SlideConfig {
    pub title: String,
    pub slide_dir: PathBuf,
    pub output_file: PathBuf,
    pub template_file: PathBuf,
    pub include_files: Option<Vec<String>>,
}

pub type ArgumentError = (String, String);

impl SlideConfig {
    pub fn proc_args(args: ArgMatches) -> Result<Self, AppError> {
        let slide_dir = args.get_one::<PathBuf>("slide_dir").ok_or_else(|| {
            ("slide_dir".to_string(),
                 "Slide directory is required".to_string())
        })?.clone();
        let output_file = args.get_one::<PathBuf>("output_file").ok_or_else(|| {
            ("output_file".to_string(), "Output file is required".to_string())
        })?.clone();
        let template_file = args.get_one::<PathBuf>("template_file").ok_or_else(|| {
            ("template_file".to_string(), "Template file is required".to_string())
        })?.clone();
        let title = args.get_one::<String>("title").ok_or_else(|| {
            ("title".to_string(), "Title is required".to_string())
        })?.clone();
        let entries = fetch_file_indices(&slide_dir)?;
        let entries = indices_and_paths_to_entries(entries)?;
        let files_to_process = build_proc_pq(entries);

        let paths = just_file_paths(&files_to_process);
        let include_files = paths.iter().map(|x| x.display().to_string());
        let include_files = Some(include_files.collect());

        Ok(Self {
            title, slide_dir, output_file, template_file, include_files
        })
    }

    pub fn validate_include_paths(&self) -> Result<(), AppError> {
        if let Some(include_files) = &self.include_files {
            for include_file in include_files {
                let file_path = self.slide_dir.join(include_file);
                validate_file_path(file_path.to_str().expect("wat the heck"))?;
            }
        }
        Ok(())
    }
}