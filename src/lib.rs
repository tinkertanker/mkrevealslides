pub mod conf;
pub mod val;
pub mod error_handling;

use std::{fs};
use std::collections::BinaryHeap;
use std::fs::File;
use std::io::{Error, ErrorKind};
use std::num::ParseIntError;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};
use tracing::{info, warn, trace};



/// Creates a directory if it does not
/// already exist
///
/// # Arguments
/// The path to the directory to create
///
/// # Errors
/// Returns an error if the directory could not be created (not because it already exists)
pub fn create_dir_if_not_exists<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    if fs::metadata(&path).is_err() {
        info!("Creating directory {} since it does not exist", path.as_ref().display());
        fs::create_dir_all(path)?
    }
    Ok(())
}

pub struct FileEntry {
    pub idx: i32,
    pub file_path: PathBuf,
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

/// Returns just the file paths given a list of FileEntries
pub fn just_file_paths(entries: &Vec<FileEntry>) -> Vec<PathBuf> {
    entries.iter().map(|e| e.file_path.clone()).collect()
}

fn is_markdown_file(fp: &Path) -> bool {
    fp.extension().unwrap_or_default() == "md"
}

/// Fetches the string of a file that comes before some seperator (in this case, the underscore)
/// This ignores directories
/// For example, if we have files:
/// - 0_intro.md
/// - 1_more.md
/// - 2a_file.md
///
/// It will grab the following indices: "0", "1", "2a"
///
/// # Arguments
/// * `path` - The path to the directory containing the files
///
/// # Errors
/// Returns an error if the directory could not be read
/// Returns an error if the file stem could not be read/converted to string/file index does not exist
///
/// # Returns
/// A vector of file indices and their paths
pub fn fetch_file_indices<P: AsRef<Path>>(dir: P) -> Result<Vec<(String, PathBuf)>, Error> {
    let inp_dir = fs::read_dir(dir)?;
    let mut files = Vec::<(String, PathBuf)>::new();
    for p in inp_dir {
        let p = p?;

        let path = p.path();
        trace!("Checking path: {}", path.display());

        if !p.file_type()?.is_file() {
            warn!("Skipping {} because it is not a file", path.display());
            continue;
        }

        if !is_markdown_file(&path) {
            warn!("Skipping {} because it is not a markdown file", path.display());
            continue;
        }

        let file_idx = path.file_stem()
            .ok_or_else(|| Error::new(ErrorKind::Other, "Could not read file stem"))?.to_str()
            .ok_or_else(|| Error::new(ErrorKind::Other, "Could not read file stem as string"))?;

        let file_idx = file_idx.split('_').collect::<Vec<&str>>();
        let file_idx = file_idx.first()
            .ok_or_else(|| Error::new(ErrorKind::Other, "Could not get file index"))?;
        files.push((file_idx.to_string(), path));

    }
    Ok(files)
}
/// Converts indices and paths to a vector of FileEntry structs
/// This will parse the index into an i32.
///
/// # Arguments
/// * `indices_and_paths` - A vector of indices and paths, generated from `fetch_file_indices`
///
/// # Returns
/// A vector of FileEntry structs
///
/// # Errors
/// Returns an error if the index could not be parsed into an i32
pub fn indices_and_paths_to_entries(indices_and_paths: Vec<(String, PathBuf)>) -> Result<Vec<FileEntry>, ParseIntError> {
    let mut entries = Vec::<FileEntry>::new();
    for (str_idx, file_path) in indices_and_paths {
        trace!("Parsing index {} for file {}", str_idx, file_path.display());
        let idx = str_idx.parse::<i32>()?;
        entries.push(FileEntry {
            idx,
            file_path,
        });
    }
    Ok(entries)
}

/// Sorts the FileEntries by their index
/// # Note
/// Time complexity: O(n log n) (heapsort)
///
/// Also completely unnecessary since `PartialOrd` is implemented for `FileEntry`
///
/// # Arguments
/// * `files` - The vector of FileEntries to sort
///
/// # Returns
/// A sorted vector of FileEntries, sorted by their index
pub fn build_proc_pq(files: Vec<FileEntry>) -> Vec<FileEntry> {
    let mut pq = BinaryHeap::new();
    // negate their idx to make a min heap
    for mut file in files {
        file.idx = -file.idx;
        pq.push(file);
    }

    let mut sorted = Vec::<FileEntry>::new();
    while let Some(mut file) = pq.pop() {
        // invert it back as we pull it out
        file.idx = -file.idx;
        sorted.push(file);
    }
    sorted
}

/// Takes a list of files, reads their contents and returns them as a vector of strings
///
/// # Arguments
/// * `files` - The vector of files to read
///
/// # Returns
/// A vector of strings, each string being the contents of a file
///
/// # Errors
/// Returns an error if even a SINGLE file could not be read
pub fn read_files_to_string(files: Vec<FileEntry>) -> Result<Vec<String>, Error> {
    let mut contents = Vec::<String>::new();
    for entry in files {
        trace!("Reading file: {}", entry.file_path.display());
        let file_contents = fs::read_to_string(entry.file_path)?;
        contents.push(file_contents);
    }
    Ok(contents)
}

/// Takes a list of ordered slide content, an input template
/// and renders the template with the slide content
///
/// # Arguments
/// * `included_slides` - The vector of strings, each string being the contents of a slide
/// * `input_template_path` - The path to the input template
/// * `presentation_title` - The title of the presentation
///
/// # Returns
/// A string, the rendered template
///
/// # Errors
/// Returns an error if the template could not be read
/// Returns an error if the template could not be rendered
pub fn gen_output_content<P: AsRef<Path>>(input_template_path: P, presentation_title: &str, included_slides: Vec<String>) -> Result<String, Error> {
    trace!("Reading template: {}", input_template_path.as_ref().display());
    trace!("Num slides: {}", included_slides.len());
    let mut ctx = Context::new();
    ctx.insert("slide_title", presentation_title);
    ctx.insert("ingested_files", &included_slides);
    let inp_template = fs::read_to_string(input_template_path)?;
    trace!("Template read. File size: {}B", inp_template.len());
    let result = Tera::one_off(&inp_template, &ctx, true);
    trace!("Template rendered. Successful: {}", result.is_ok());
    result.map_err(|e| Error::new(ErrorKind::Other, e.to_string()))
}
