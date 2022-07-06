use std::path::PathBuf;

/// Useful validators for clap arguments

pub type ValError = String;

/// Checks whether the given path exists and is a file
pub fn validate_file_path(s: &str) -> Result<(), ValError> {
    let pb = PathBuf::from(s);
    if !pb.exists() {
        return Err(format!("File {} does not exist", pb.display()));
    }
    if pb.is_file() {
         Ok(())
    } else {
        Err(format!("{} is not a file", s))
    }
}

/// Checks whether the given path is a directory and if it exists
pub fn validate_dir_path(s: &str) -> Result<(), ValError> {
    let pb = PathBuf::from(s);
    if !pb.exists() {
        return Err(format!("Directory {} does not exist", pb.display()));
    }
    if pb.is_dir() {
         Ok(())
    } else {
        Err(format!("{} is not a directory", s))
    }
}