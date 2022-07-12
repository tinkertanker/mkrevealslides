use std::cmp::Ordering;
use std::fs;

use std::io::Error;

use std::path::{Path, PathBuf};
use anyhow::Context;

use tracing::trace;

/// A SlideFile is a slide that exists as a file on the disk somewhere
#[derive(PartialEq, Debug, Clone)]
pub struct SlideFile {
    filename: String,
    /// Absolute path to where this slideFile is located
    pub path: PathBuf,
}

impl PartialOrd for SlideFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.filename.partial_cmp(&other.filename)
    }
}

impl Ord for SlideFile {
    fn cmp(&self, other: &Self) -> Ordering {
        self.filename.cmp(&other.filename)
    }
}

impl Eq for SlideFile {}

impl TryFrom<PathBuf> for SlideFile {
    type Error = anyhow::Error;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let filename = path
            .file_name()
            .with_context(|| format!("`{}` does not contain a valid filename", path.display()))?
            .to_str()
            .with_context(|| format!("Filename at `{}` is not UTF-8!", path.display()))?
            .to_string();
        let sf = Self {
            filename,
            path,
        };
        sf.validate()?;
        Ok(sf)
    }
}

impl SlideFile {
    /// Creates a list of SlideFiles from paths
    /// # Arguments
    /// * `paths` - A list of paths to slide files.
    ///
    /// # Returns
    /// A list of SlideFiles.
    ///
    /// # Errors
    /// - If a slide file has an invalid file name
    /// - If a slide file has a filename that is not UTF-8 compatible
    fn from_paths(paths: Vec<PathBuf>) -> Result<Vec<Self>, anyhow::Error> {
        paths
            .into_iter()
            .map(SlideFile::try_from)
            .collect::<Result<Vec<SlideFile>, anyhow::Error>>()
    }

    /// Attempts to validate the SlideFile
    /// This checks if the file
    /// - actually exists
    /// - can be read
    /// - is a .md file
    ///
    /// # Returns
    /// None
    ///
    /// # Errors
    /// - If the slide file does not exist
    /// - If the slide file is not a file
    /// - If the slide file is not a markdown file
    pub fn validate(&self) -> Result<(), anyhow::Error> {
        // todo: return ValidationError
        if !self.path.is_absolute() {
            return Err(anyhow::Error::msg(format!("Path `{}` is not absolute!", self.path.display())));
        }
        if !self.path.exists() {
            return Err(anyhow::Error::msg(format!("File at `{}` does not exist!", self.path.display())));
        }
        if !self.path.is_file() {
            return Err(anyhow::Error::msg(format!("File at `{}` is not a file!", self.path.display())));
        }
        if !is_markdown_file(&self.path) {
            return Err(anyhow::Error::msg(format!("File at `{}` is not a markdown file!", self.path.display())));
        }
        Ok(())
    }
}

/// Checks if the file at the given path has an extension of .md
pub fn is_markdown_file(fp: &Path) -> bool {
    fp.extension().unwrap_or_default().to_ascii_lowercase() == "md"
}

/// Attempts to find slides in the given directory
///
/// # Arguments
/// * slide_dir: The directory that contains your slides
///
/// # Returns
/// A sorted vector of paths to slides in the given directory, sorted by alphabetical order
///
/// # Errors
/// Returns an error if the slide directory could not be read
pub fn find_slides(slide_dir: &PathBuf) -> Result<Vec<SlideFile>, anyhow::Error> {
    trace!("Finding slides in {}", slide_dir.display());
    let files = list_directory(slide_dir, true)?;
    let mut slide_files = SlideFile::from_paths(files)?;
    slide_files.sort();
    for slide_file in &slide_files {
        slide_file.validate()?;
    }
    Ok(slide_files)
}

/// Lists a given directory
/// # Arguments
/// * path: The directory to list
/// * only_files: Whether to only return files or not
///
/// # Returns
/// A vector of paths to things in that directory
fn list_directory<Pth: AsRef<Path>>(path: Pth, only_files: bool) -> Result<Vec<PathBuf>, Error> {
    let mut paths: Vec<PathBuf> = Vec::new();
    let dir = fs::read_dir(path.as_ref())?;
    for entry in dir {
        let entry = entry?;
        let path = entry.path();
        if only_files {
            if path.is_file() {
                paths.push(path);
            }
        } else {
            paths.push(path);
        }
    }
    Ok(paths)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_is_markdown_file() {
        let md_file_name = PathBuf::from("/a/b/c/file.md");
        let not_md_file_name = PathBuf::from("/a/b/c/file.txt");
        let definitely_not_md = PathBuf::from("/a/b/c/file");
        assert!(is_markdown_file(&md_file_name));
        assert!(!is_markdown_file(&not_md_file_name));
        assert!(!is_markdown_file(&definitely_not_md));
    }

    #[test]
    fn test_find_included_slides() {
        let slides_dir = tempdir().unwrap();
        let slides_dir = fs::canonicalize(slides_dir.path()).unwrap();
        let slide_file_1 = slides_dir.join("1_slide1.md");
        let slide_file_2 = slides_dir.join("2_slide2.md");
        let slide_file_3 = slides_dir.join("3_slide3.md");
        File::create(&slide_file_1).unwrap();
        File::create(&slide_file_2).unwrap();
        File::create(&slide_file_3).unwrap();
        let slides = find_slides(&slides_dir).unwrap();
        assert_eq!(slides, vec![SlideFile{
            filename: "1_slide1.md".to_string(),
            path: slide_file_1,
        }, SlideFile{
            filename: "2_slide2.md".to_string(),
            path: slide_file_2,
        }, SlideFile{
            filename: "3_slide3.md".to_string(),
            path: slide_file_3,
        }]);
    }

    #[test]
    fn test_find_included_slides_fails() {
        let slides_dir = tempdir().unwrap();
        let good_slide_file = slides_dir.path().join("1_slide1.md");
        let bad_slide_file = slides_dir.path().join("slide2_2.txt");
        File::create(&good_slide_file).unwrap();
        File::create(&bad_slide_file).unwrap();
        let slides = find_slides(&slides_dir.into_path());
        assert!(slides.is_err());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_grab_file_names_from_path_bufs_windows() {
        let paths = vec![
            PathBuf::from(r"C:\Users\file4.txt"),
            PathBuf::from(r"C:\file_no_ext"),
        ];
        let file_names = grab_file_names_from_path_bufs(&paths).unwrap();
        assert_eq!(
            file_names,
            vec![PathBuf::from("file4.txt"), PathBuf::from("file_no_ext")]
        );
    }
}