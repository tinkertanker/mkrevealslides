pub mod parsing;

use crate::errors::ValidationError;
use anyhow::Context;
use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

use crate::presentation::io::is_markdown_file;
use crate::presentation::slide::parsing::{get_local_links, grab_image_links};

/// A SlideFile is a slide that exists as a file on the disk somewhere
#[derive(PartialEq, Debug, Clone)]
pub struct SlideFile {
    filename: String,
    /// Absolute path to where this slideFile is located on the disk
    pub path: PathBuf,
    /// Full contents of the SlideFile
    pub contents: String,

    pub local_images: Vec<PathBuf>,
}

impl PartialOrd for SlideFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // todo: natural sorting
        self.filename.partial_cmp(&other.filename)
    }
}

impl Ord for SlideFile {
    fn cmp(&self, other: &Self) -> Ordering {
        self.filename.cmp(&other.filename)
    }
}

impl Eq for SlideFile {}

impl SlideFile {
    /// Reads a SlideFile from the disk.
    /// This will also transform any local links to be relative to <OUTPUT_DIR>/img/
    ///
    /// For example, say you have a slide with the following content
    /// ```markdown
    /// ![](../whatever/image.png)
    /// ```
    /// When read in, the `content` of this SlideFile will contain instead
    /// ```markdown
    /// ![](./img/whatever/image.png)
    /// ```
    ///
    ///
    /// # Arguments
    /// * `path` - Absolute path to the SlideFile on the disk
    ///
    /// # Errors
    /// * `ValidationError` - If the SlideFile is not a valid SlideFile
    /// * `std::io::Error` - If there was an error reading the SlideFile
    ///
    /// # Notes
    /// This is a blocking operation since it will read the file from the disk
    /// and attempt to parse it.
    pub fn read_and_parse<P: AsRef<Path>>(path: P) -> Result<Self, anyhow::Error> {
        let filename = path
            .as_ref()
            .file_name()
            .with_context(|| {
                format!(
                    "`{}` does not contain a valid filename",
                    path.as_ref().display()
                )
            })?
            .to_str()
            .with_context(|| format!("Filename at `{}` is not UTF-8!", path.as_ref().display()))?
            .to_string();
        SlideFile::validate_path(&path)?;
        let contents = fs::read_to_string(&path)?;

        let local_images = get_local_links(&grab_image_links(&contents));

        let sf = Self {
            filename,
            path: path.as_ref().to_path_buf(),
            contents,
            local_images,
        };
        Ok(sf)
    }

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
    pub fn from_paths(paths: Vec<PathBuf>) -> Result<Vec<Self>, anyhow::Error> {
        paths
            .into_iter()
            .map(SlideFile::read_and_parse)
            .collect::<Result<Vec<SlideFile>, anyhow::Error>>()
    }

    /// Attempts to validate the path to a SlideFile
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
    fn validate_path<P: AsRef<Path>>(slide_file_path: P) -> Result<(), ValidationError> {
        if !slide_file_path.as_ref().is_absolute() {
            return Err(ValidationError::new(
                &slide_file_path.as_ref().display().to_string(),
                "Path is not absolute".to_string(),
            ));
        }
        if !slide_file_path.as_ref().exists() {
            return Err(ValidationError::new(
                &slide_file_path.as_ref().display().to_string(),
                "File does not exist".to_string(),
            ));
        }
        if !slide_file_path.as_ref().is_file() {
            return Err(ValidationError::new(
                &slide_file_path.as_ref().display().to_string(),
                "Path is not a file".to_string(),
            ));
        }
        if !is_markdown_file(slide_file_path.as_ref()) {
            return Err(ValidationError::new(
                &slide_file_path.as_ref().display().to_string(),
                "File is not a markdown file".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_parse_slide() {
        let slide_contents = r#"![oh no an image](./local/image.png)"#;
        let tmp_dir = tempdir().unwrap();
        let abs_path_to_tmp_dir = fs::canonicalize(tmp_dir.path()).unwrap();
        let slide_file = abs_path_to_tmp_dir.join("slide.md");
        let mut h_slide_file = File::create(&slide_file).unwrap();
        h_slide_file.write_all(slide_contents.as_bytes()).unwrap();

        let local_img = abs_path_to_tmp_dir.join("local/image.png");
        fs::create_dir_all(local_img.parent().unwrap()).unwrap();
        let _h_local_img = File::create(&local_img).unwrap();

        let slide_file = SlideFile::read_and_parse(slide_file).unwrap();
        assert_eq!(slide_file.contents, slide_contents);
        assert_eq!(slide_file.local_images.len(), 1);
        assert_eq!(
            slide_file.local_images[0],
            PathBuf::from("/haha/local/image.png")
        );
    }
}
