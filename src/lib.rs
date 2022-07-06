use std::{fs, io};
/// Creates a directory if it does not
/// already exist
///
/// # Arguments
/// The path to the directory to create
///
/// # Errors
/// Returns an error if the directory could not be created (not because it already exists)
pub fn create_dir_if_not_exists(path: &str) -> Result<(), io::Error> {
    if fs::metadata(path).is_err() {
        fs::create_dir_all(path)?
    }
    Ok(())
}

pub struct FileEntry {
    pub idx: i32,
    pub file_path: String
}

impl PartialOrd for FileEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.idx.cmp(&other.idx))
    }
}

impl PartialEq for FileEntry {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx
    }
}

impl Ord for FileEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.idx.cmp(&other.idx)
    }
}

impl Eq for FileEntry {}