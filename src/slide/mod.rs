pub mod io;
pub mod parsing;

use std::fs;
use std::path::PathBuf;

use crate::slide::io::SlideFile;

use crate::slide::parsing::{get_local_links, grab_image_links};

/// A single slide that can be used in a presentation.
/// A slide has contents (which is in markdown)
/// It may also contain local images
///
/// # Note
/// Constructing a slide with contents does not automatically fill in local_images, and
/// instead the `parse` method must be used to do so
// todo: consider automatically parsing local images when reading a slide
// todo: investigate whether adding serialize and deserialize is better than having a render method.
#[derive(Debug)]
pub struct Slide {
    pub contents: String,
    /// Path to the slide file.
    /// May be `None` if the slide was not read from a file.
    pub slide_path: Option<PathBuf>,
    /// `None` means that `contents` has not been parsed yet
    /// If this is `Some`, the local_images may still be empty if there are no local images
    pub local_images: Option<Vec<String>>,
}

impl TryFrom<SlideFile> for Slide {
    type Error = anyhow::Error;

    /// This consumes the SlideFile
    fn try_from(slide_file: SlideFile) -> Result<Self, Self::Error> {
        let contents = fs::read_to_string(&slide_file.path)?;
        Ok(Self {
            contents,
            slide_path: Some(slide_file.path),
            local_images: None,
        })
    }
}

impl Slide {
    /// Creates a new `Slide` from contents
    /// This does not parse the contents automatically.
    pub fn new(contents: String) -> Self {
        Self {
            contents,
            slide_path: None,
            local_images: None,
        }
    }

    /// Parses the contents of the slide, looking for
    /// local images and fills in the `local_images` field
    pub fn parse(&mut self) {
        let im_links = grab_image_links(&self.contents);
        let local_links = get_local_links(im_links);
        self.local_images = Some(local_links);
    }

    /// Renders the slide
    ///
    /// # Note
    /// This actually just clones the contents, which is technically inefficient.
    /// We will see if this becomes a problem.
    pub fn render(&self) -> String {
        self.contents.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_slide_from_file() {
        let slide_contents = r#"
# Hello World

This is a markdown slide, used for testing.
        "#;
        let tmp_dir = tempdir().unwrap();
        let slide_file = tmp_dir.path().join("slide.md");
        let mut h_slide_file = File::create(&slide_file).unwrap();
        h_slide_file.write_all(slide_contents.as_bytes()).unwrap();
        let s_slidefile = SlideFile::try_from(slide_file).unwrap();
        let slide = Slide::try_from(s_slidefile).unwrap();
        assert_eq!(slide.contents, slide_contents);
    }

    #[test]
    fn test_slide_creation_does_not_auto_parse_images() {
        let slide_contents = "Cool little slide eh ![oh no an image](/haha/local/image.png)";
        let slide = Slide::new(slide_contents.to_string());
        assert_eq!(slide.local_images, None);
    }

    #[test]
    fn test_slide_parsing() {
        let slide_contents = "Cool little slide eh ![oh no an image](/haha/local/image.png)";
        let mut slide = Slide::new(slide_contents.to_string());
        slide.parse();
        assert_eq!(slide.local_images.unwrap()[0], "/haha/local/image.png");
    }
}
