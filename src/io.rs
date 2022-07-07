use std::fs;

use std::io::{Error, ErrorKind};
use std::num::ParseIntError;

use std::path::{Path, PathBuf};

use crate::error_handling::AppError;
use tracing::{error, trace, warn};

#[derive(Clone, Debug)]
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

/// Checks if the file at the given path has an extension of .md
fn is_markdown_file(fp: &Path) -> bool {
    fp.extension().unwrap_or_default().to_ascii_lowercase() == "md"
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
    trace!("Fetching file indices from directory: {}", dir.as_ref().display());
    let inp_dir = fs::read_dir(dir);
    let inp_dir = match inp_dir {
        Ok(dir) => dir,
        Err(e) => {
            error!("Could not read directory: {}", e);
            return Err(e);
        },
    };
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
            warn!(
                "Skipping {} because it is not a markdown file",
                path.display()
            );
            continue;
        }

        let file_idx = path
            .file_stem()
            .ok_or_else(|| Error::new(ErrorKind::Other, "Could not read file stem"))?
            .to_str()
            .ok_or_else(|| Error::new(ErrorKind::Other, "Could not read file stem as string"))?;

        let file_idx = file_idx.split('_').collect::<Vec<&str>>();
        let file_idx = file_idx
            .first()
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
pub fn indices_and_paths_to_entries(
    indices_and_paths: Vec<(String, PathBuf)>,
) -> Result<Vec<FileEntry>, ParseIntError> {
    let mut entries = Vec::<FileEntry>::new();
    for (str_idx, file_path) in indices_and_paths {
        trace!("Parsing index {} for file {}", str_idx, file_path.display());
        let idx = str_idx.parse::<i32>()?;
        entries.push(FileEntry { idx, file_path });
    }
    Ok(entries)
}

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
pub fn find_slides(slide_dir: &PathBuf) -> Result<Vec<PathBuf>, AppError> {
    trace!("Finding slides in {}", slide_dir.display());
    let mut included_slides = Vec::new();
    let entries = fetch_file_indices(slide_dir)?;
    let mut entries = indices_and_paths_to_entries(entries)?;
    entries.sort();
    for entry in entries {
        included_slides.push(entry.file_path.clone());
    }
    Ok(included_slides)
}

/// Takes a list of file paths, and only returns their file names (with extensions)
pub fn grab_file_names_from_path_bufs(paths: &Vec<PathBuf>) -> Result<Vec<PathBuf>, AppError> {
    let mut file_names = Vec::new();
    for path in paths {
        let file_name = path
            .file_name()
            .ok_or_else(|| AppError::new("Could not get file name from path"))?;
        let file_name = PathBuf::from(file_name);
        file_names.push(file_name);
    }
    Ok(file_names)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    /// Creates a String from a &str
    macro_rules! hs {
        ($s:expr) => {
            String::from($s)
        };
    }

    #[test]
    fn test_file_entry_sortable() {
        let mut file_entries = vec![
            FileEntry {
                idx: 3,
                file_path: PathBuf::from("/tmp/3.md"),
            },
            FileEntry {
                idx: 1,
                file_path: PathBuf::from("/tmp/1.md"),
            },
            FileEntry {
                idx: 2,
                file_path: PathBuf::from("/tmp/2.md"),
            },
        ];
        file_entries.sort();
        assert_eq!(file_entries[0].idx, 1);
        assert_eq!(file_entries[1].idx, 2);
        assert_eq!(file_entries[2].idx, 3);
    }

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
    fn test_fetch_file_indices() {
        let tmp_dir = tempdir().unwrap();
        let file_1 = tmp_dir.path().join("0_test.md");
        let file_2 = tmp_dir.path().join("1_test.md");
        let file_3 = tmp_dir.path().join("2_test.md");

        let file_paths = vec![file_1, file_2, file_3];
        for file_path in &file_paths {
            File::create(file_path).unwrap();
        }

        let indices_and_paths = fetch_file_indices(tmp_dir.path()).unwrap();

        let just_indices: Vec<String> = indices_and_paths
            .iter()
            .map(|(idx, _)| idx.clone())
            .collect();
        assert_eq!(indices_and_paths.len(), 3);
        assert!(just_indices.contains(&"0".to_string()));
        assert!(just_indices.contains(&"1".to_string()));
        assert!(just_indices.contains(&"2".to_string()));

        tmp_dir.close().unwrap();
    }

    #[test]
    fn test_fetch_file_indices_skips() {
        let tmp_dir = tempdir().unwrap();
        let md_file_1 = tmp_dir.path().join("0_test.md");
        let md_file_2 = tmp_dir.path().join("1_test.md");
        let md_file_3 = tmp_dir.path().join("2_test.md");
        let dir = tmp_dir.path().join("not_md_file");
        let not_md_file = tmp_dir.path().join("not_md_file.txt");

        File::create(&md_file_1).unwrap();
        File::create(&md_file_2).unwrap();
        File::create(&md_file_3).unwrap();
        fs::create_dir(&dir).unwrap();
        File::create(&not_md_file).unwrap();

        let indices_and_paths = fetch_file_indices(tmp_dir.path()).unwrap();
        assert_eq!(indices_and_paths.len(), 3);
    }

    #[test]
    fn test_fetch_weird_indices() {
        // yes, this should work
        let tmp_dir = tempdir().unwrap();
        let file_1 = tmp_dir.path().join("test0.md");
        let file_2 = tmp_dir.path().join("test1.md");
        let file_3 = tmp_dir.path().join("test2.md");

        let file_paths = vec![file_1, file_2, file_3];
        for file_path in file_paths {
            File::create(file_path).unwrap();
        }

        let indices_and_paths = fetch_file_indices(tmp_dir.path());
        assert!(indices_and_paths.is_ok());

        tmp_dir.close().unwrap();
    }

    #[test]
    fn test_fetch_bad_indices() {
        let tmp_dir = tempdir().unwrap();
        let file_1 = tmp_dir.path().join("0_test.md");
        File::create(&file_1).unwrap();

        let indices_and_paths = fetch_file_indices(file_1.as_path());
        assert!(indices_and_paths.is_err());

        tmp_dir.close().unwrap();
    }

    #[test]
    fn test_fetch_bad_indices_2() {
        // This will result in an empty result since there are no md files
        let tmp_dir = tempdir().unwrap();
        let dir_1 = tmp_dir.path().join("0_test");
        let dir_2 = tmp_dir.path().join("1_test");
        let dir_3 = tmp_dir.path().join("2_test");
        fs::create_dir(&dir_1).unwrap();
        fs::create_dir(&dir_2).unwrap();
        fs::create_dir(&dir_3).unwrap();

        let indices_and_paths = fetch_file_indices(tmp_dir.path());
        assert!(indices_and_paths.unwrap().is_empty());

        tmp_dir.close().unwrap();
    }

    #[test]
    fn test_indices_and_paths_to_entries() {
        let indices_and_paths = vec![
            (hs!("1"), PathBuf::from("/a/b/c/file.md")),
            (hs!("2"), PathBuf::from("/a/b/c/file2.md")),
            (hs!("3"), PathBuf::from("/a/b/c/file3.md")),
        ];
        let mut entries = indices_and_paths_to_entries(indices_and_paths).unwrap();
        assert_eq!(entries.len(), 3);
        entries.sort();
        assert_eq!(entries[0].file_path, PathBuf::from("/a/b/c/file.md"));
        assert_eq!(entries[1].file_path, PathBuf::from("/a/b/c/file2.md"));
        assert_eq!(entries[2].file_path, PathBuf::from("/a/b/c/file3.md"));
    }

    #[test]
    fn test_bad_indices_and_paths_to_entires() {
        let bad_indices_and_paths =
            vec![(hs!("not int"), PathBuf::from("/doesnt/really/matter/lol"))];
        let entries = indices_and_paths_to_entries(bad_indices_and_paths);
        assert!(entries.is_err());
    }

    #[test]
    fn test_grab_file_names_from_path_bufs() {
        let paths = vec![
            PathBuf::from("/path/to/file1.txt"),
            PathBuf::from("../relative/path/to/file2.md"),
            PathBuf::from("./path/to/file3.html"),
            PathBuf::from("file_no_ext"),
        ];
        let file_names = grab_file_names_from_path_bufs(&paths).unwrap();
        assert_eq!(
            file_names,
            vec![
                PathBuf::from("file1.txt"),
                PathBuf::from("file2.md"),
                PathBuf::from("file3.html"),
                PathBuf::from("file_no_ext"),
            ]
        );
    }

    #[test]
    fn test_find_included_slides() {
        let slides_dir = tempdir().unwrap();
        let slide_file_1 = slides_dir.path().join("1_slide1.md");
        let slide_file_2 = slides_dir.path().join("2_slide2.md");
        let slide_file_3 = slides_dir.path().join("3_slide3.md");
        let not_md_file = slides_dir.path().join("5_not_md.txt");
        File::create(&slide_file_1).unwrap();
        File::create(&slide_file_2).unwrap();
        File::create(&slide_file_3).unwrap();
        File::create(&not_md_file).unwrap();
        let slides = find_slides(&slides_dir.into_path()).unwrap();
        assert_eq!(slides, vec![slide_file_1, slide_file_2, slide_file_3]);
    }

    #[test]
    fn test_find_included_slides_fails() {
        let slides_dir = tempdir().unwrap();
        let good_slide_file = slides_dir.path().join("1_slide1.md");
        let bad_slide_file = slides_dir.path().join("slide2_2.md");
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
            vec![
                PathBuf::from("file4.txt"),
                PathBuf::from("file_no_ext")
            ]
        );
    }
}
