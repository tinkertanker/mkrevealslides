use crate::errors::ValidationError;
use anyhow::Context;
use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};
use pulldown_cmark::{Event, html, Options, Parser, Tag};

use crate::presentation::io::is_markdown_file;

/// A SlideFile is a slide that exists as a file on the disk somewhere
#[derive(PartialEq, Debug, Clone)]
pub struct SlideFile {
    filename: String,
    /// Absolute path to where this slideFile is located on the disk
    pub path: PathBuf,
    /// Full contents of the SlideFile
    pub contents: String,

    pub local_images: Vec<(PathBuf, PathBuf)>,
}

impl PartialOrd for SlideFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(natord::compare(&self.filename, &other.filename))
    }
}

impl Ord for SlideFile {
    fn cmp(&self, other: &Self) -> Ordering {
        natord::compare(&self.filename, &other.filename)
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

        let path = path.as_ref().to_path_buf();
        let filename = path
            .file_name()
            .with_context(|| {
                format!(
                    "`{}` does not contain a valid filename",
                    path.display()
                )
            })?
            .to_str()
            .with_context(|| format!("Filename at `{}` is not UTF-8!", path.display()))?
            .to_string();
        Self::validate_path(&path)?;
        let contents = fs::read_to_string(&path)?;
        let mut local_images = Vec::new();

        let parser = Parser::new_ext(&contents, Options::all());
        let parser = parser.map(|event| match event {
            Event::Start(Tag::Image(link_type, url, title)) => {
                // check if the image is local
                if !url.contains("://") {
                    let img_path = PathBuf::from(url.as_ref());
                    let img_abs_path = if !img_path.is_absolute() {
                        let img_abs_path = fs::canonicalize(path.parent()
                            .expect("slide file to have parent")
                            .join(img_path))
                            .expect("img path to exist");
                        img_abs_path
                    } else {
                        img_path
                    };
                    // this is a local image, let's grab the full path to it
                    let img_filename = img_abs_path.file_name()
                        .expect("image to have a valid file name");
                    // todo: this will BREAK if there are other images with the same name, best to use a hash
                    // the destination path is ./img/<slide filename>/<img filename>
                    let dst_path = PathBuf::from("./img")
                        .join(&filename)
                        .join(img_filename)
                        .to_str().expect("can convert to string").to_string();
                    local_images.push((img_abs_path, PathBuf::from(&dst_path)));
                    Event::Start(Tag::Image(link_type, dst_path.into(), title))
                } else {
                    // don't rewrite the link
                    Event::Start(Tag::Image(link_type, url, title))
                }
            },
            _ => event
        });

        let mut contents = String::new();
        html::push_html(&mut contents, parser);

        let sf = Self {
            filename,
            path,
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
        assert_eq!(slide_file.contents, "<p><img src=\"./img/slide.md/image.png\" alt=\"oh no an image\" /></p>\n");
        assert_eq!(slide_file.local_images.len(), 1);
        assert_eq!(
            slide_file.local_images[0],
            (local_img, PathBuf::from("./img/slide.md/image.png"))
        );
    }
}
