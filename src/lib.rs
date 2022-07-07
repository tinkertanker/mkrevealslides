pub mod conf;
pub mod val;
pub mod error_handling;
pub mod parsing;
pub mod presentation;
pub mod slide;

use std::{fs};
use std::collections::BinaryHeap;
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

#[derive(Clone)]
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
pub fn just_file_paths(entries: &[FileEntry]) -> Vec<PathBuf> {
    entries.iter().map(|e| e.file_path.clone()).collect()
}

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
pub fn read_files_to_string(files: &Vec<PathBuf>) -> Result<Vec<String>, Error> {
    let mut contents = Vec::<String>::new();
    for entry in files {
        trace!("Reading file: {}", entry.display());
        let file_contents = fs::read_to_string(entry)?;
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
pub fn gen_output_content<P: AsRef<Path>>(input_template_path: P, presentation_title: &str,
                                          included_slides: Vec<String>) -> Result<String, Error> {
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


#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::Write;
    use std::iter::zip;
    use tempfile::{tempdir};
    use super::*;

    /// Creates a String from a &str
    macro_rules! hs {
        ($s:expr) => {
            String::from($s)
        }
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
        let proc_pq = build_proc_pq(file_entries.clone());
        file_entries.sort();
        assert_eq!(file_entries[0].idx, 1);
        assert_eq!(file_entries[1].idx, 2);
        assert_eq!(file_entries[2].idx, 3);

        assert_eq!(proc_pq[0].idx, 1);
        assert_eq!(proc_pq[1].idx, 2);
        assert_eq!(proc_pq[2].idx, 3);
    }

    #[test]
    fn test_is_markdown_file() {
        let md_file_name = PathBuf::from("/a/b/c/file.md");
        let not_md_file_name = PathBuf::from("/a/b/c/file.txt");
        let definitely_not_md  = PathBuf::from("/a/b/c/file");
        assert!(is_markdown_file(&md_file_name));
        assert!(!is_markdown_file(&not_md_file_name));
        assert!(!is_markdown_file(&definitely_not_md));
    }

    #[test]
    fn test_just_file_paths() {
        let entries = vec![
            FileEntry {
                idx: 1,
                file_path: PathBuf::from("/a/b/c/file.md"),
            },
            FileEntry {
                idx: 2,
                file_path: PathBuf::from("/a/b/c/file2.md"),
            },
        ];

        let just_file_paths = just_file_paths(&entries);
        assert_eq!(just_file_paths.len(), 2);
        assert_eq!(just_file_paths[0], PathBuf::from("/a/b/c/file.md"));
        assert_eq!(just_file_paths[1], PathBuf::from("/a/b/c/file2.md"));
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

        let just_indices: Vec<String> = indices_and_paths.iter().map(|(idx, _)| idx.clone()).collect();
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
    fn test_gen_output_content() {
        let tmp_dir = tempdir().unwrap();
        let template_content = "{{ slide_title }} {%for fc in ingested_files %}'{{fc}}'{%endfor%}";
        let mut tmp_template = tmp_dir.path().join("template.tera");
        File::create(&mut tmp_template).unwrap().write_all(template_content.as_bytes()).unwrap();
        let slide_contents = vec![hs!("a"), hs!("b"), hs!("c")];
        let output_content = gen_output_content(tmp_template,
                                                "test",
                                                slide_contents).unwrap();
        assert_eq!(output_content, "test 'a''b''c'");
    }

    #[test]
    fn test_read_files_to_string() {
        let tmp_dir = tempdir().unwrap();
        let file_1 = tmp_dir.path().join("0_test.md");
        let file_2 = tmp_dir.path().join("1_test.md");
        let file_3 = tmp_dir.path().join("2_test.md");

        let file_paths = vec![file_1, file_2, file_3];
        let expected_contents = vec!["file_1", "file_2", "file_3"];

        for (fp, content) in zip(&file_paths, &expected_contents) {
            let mut created_file = File::create(&fp).unwrap();
            created_file.write_all(content.as_bytes()).unwrap();
        }

        let mut contents = read_files_to_string(&file_paths).unwrap();
        contents.sort();
        assert_eq!(contents.len(), 3);
        assert_eq!(contents, expected_contents);
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
        let bad_indices_and_paths = vec![
            (hs!("not int"), PathBuf::from("/doesnt/really/matter/lol"))
        ];
        let entries = indices_and_paths_to_entries(bad_indices_and_paths);
        assert!(entries.is_err());
    }
}