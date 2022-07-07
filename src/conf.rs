use std::path::PathBuf;
use clap::ArgMatches;
use serde::{Deserialize};
use crate::{build_proc_pq, fetch_file_indices, indices_and_paths_to_entries};
use crate::error_handling::AppError;
use crate::val::validate_file_path;

#[derive(Debug, Deserialize, Clone)]
pub struct SlideConfig {
    pub title: String,
    pub slide_dir: PathBuf,
    pub output_file: PathBuf,
    pub template_file: PathBuf,
    pub include_files: Option<Vec<String>>,
}

pub type ArgumentError = (String, String);

/// Walks the given directory and returns the slides found in there
///
/// # Arguments
/// * slide_dir: The directory that contains your slides
///
/// # Returns
/// A vector of paths to the slides, sorted.
///
/// # Errors
/// Returns an error if the slide directory could not be read
/// Returns an error if the indices could not be converted
pub fn find_included_slides(slide_dir: &PathBuf) -> Result<Vec<PathBuf>, AppError> {
    let mut included_slides = Vec::new();
    let entries = fetch_file_indices(slide_dir)?;
    let entries = indices_and_paths_to_entries(entries)?;
    let entries = build_proc_pq(entries);
    for entry in entries {
        included_slides.push(entry.file_path.clone());
    }
    Ok(included_slides)
}

/// Takes a list of file paths, and only returns their file names (with extensions)
fn grab_file_names_from_pathbufs(paths: &Vec<PathBuf>) -> Result<Vec<String>, AppError> {
    let mut file_names = Vec::new();
    for path in paths {
        let file_name = path.file_name().ok_or_else(|| {
            AppError::new("Could not get file name from path")
        })?.to_str().ok_or_else(|| {
            AppError::new("Could not get file name as string")
        })?.to_string();
        file_names.push(file_name);
    }
    Ok(file_names)
}

impl SlideConfig {
    /// Processes arguments provided to the program
    /// and builds the logical configuration from that
    ///
    /// # Errors
    /// Returns error if the arguments are invalid (e.g. missing required arguments)
    /// Returns error if it could not parse the file indices into a vector of FileEntry structs
    /// Returns error if it could not read the template file
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

        let include_files = find_included_slides(&slide_dir)?;
        let include_files = grab_file_names_from_pathbufs(&include_files)?;

        Ok(Self {
            title, slide_dir, output_file, template_file, include_files: Some(include_files),
        })
    }

    /// Validates the include_files field checking if files actually exist.
    /// The validation only runs if the include_files field is None,
    /// otherwise, the program will automatically search for files.
    pub fn validate_include_paths(&self) -> Result<(), AppError> {
        if let Some(include_files) = &self.include_files {
            for include_file in include_files {
                let file_path = self.slide_dir.join(include_file);
                validate_file_path(file_path.to_str().ok_or_else(|| {
                    AppError::new("Could not convert file path to string")
                })?)?;
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