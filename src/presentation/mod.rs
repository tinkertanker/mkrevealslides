use slide::Slide;
use crate::ui::PresentationConfig;
use anyhow::Context;
use std::fs;
use std::path::{Path, PathBuf};
use tera::Tera;
use tracing::{debug, trace, warn};

pub mod slide;

#[derive(Debug)]
pub struct Presentation {
    /// The title of the presentation
    pub title: String,
    /// Contains the contents of the template to use for the presentation
    pub template: String,
    /// Contains the slides, in order, to include in the presentation
    pub slides: Vec<Slide>,
}

/// Attempts to parse the PresentationConfig and read all the necessary details in
/// producing a presentation
impl TryFrom<PresentationConfig> for Presentation {
    type Error = anyhow::Error;

    /// Attempts to parse the PresentationConfig and read all the necessary details in
    /// producing a presentation
    ///
    /// # Arguments
    /// * `config` - The PresentationConfig to parse
    ///
    /// # Returns
    /// A Presentation if the config is valid
    ///
    /// # Errors
    /// - If the slides could not be read
    /// - If the template could not be read
    fn try_from(config: PresentationConfig) -> Result<Self, Self::Error> {
        trace!("Attempting to parse PresentationConfig");
        trace!("Presentation title: {}", &config.title);
        trace!(
            "Reading template_file at `{}`",
            &config.output_file.display()
        );
        let template = fs::read_to_string(&config.template_file)?;
        trace!("Template file read: {} bytes", template.len());
        trace!("Reading {} slides", &config.include_files.len());
        let slides = {
            let mut slides = Vec::new();
            for slide_file in config.include_files {
                trace!("Reading slide at `{}`", slide_file.path.display());
                let slide = Slide::try_from(slide_file)?;
                slides.push(slide);
            }
            slides
        };
        trace!("Parsed {} slides", slides.len());
        Ok(Self {
            title: config.title,
            template,
            slides,
        })
    }
}

impl Presentation {
    /// Renders the presentation into a string
    ///
    /// # Returns
    /// Returns the contents of the presentation as a String
    ///
    /// # Errors
    /// If the template engine fails to render the presentation.
    pub fn render(&self) -> Result<String, tera::Error> {
        let mut ctx = tera::Context::new();

        let slide_contents = self
            .slides
            .iter()
            .map(|s| s.render())
            .collect::<Vec<String>>();
        ctx.insert("slide_title", &self.title);
        ctx.insert("ingested_files", &slide_contents);

        let result = Tera::one_off(&self.template, &ctx, false);
        trace!("Render template succeeded: {}", result.is_ok());
        result
    }

    /// Packages the presentation to a file.
    /// This will copy all local images referenced in slides into the output directory
    ///
    /// # Arguments
    /// * `output_dir`: The directory to place all the presentation output files into
    ///
    /// Optionally, downloads revealJS libs and generates the zip too
    pub fn package<P: AsRef<Path>>(&mut self, output_dir: P) -> Result<(), anyhow::Error> {
        // todo: refactor logic here, too messy
        let output = self.render()?;
        debug!("Rendered {} bytes", output.len());
        // todo: read the config!
        let output_path = output_dir.as_ref().join("index.html");
        debug!("Writing to `{}`", output_path.display());
        fs::write(&output_path, output)?;
        println!("Slides written to `{}`", output_path.display());

        for slide in &mut self.slides {
            if slide.slide_path.is_none() {
                // todo: support images with absolute paths
                warn!("Skipping a slide which has no path");
                continue;
            }
            slide.parse();
            let slide_path = slide.slide_path.as_ref().unwrap();

            trace!("Slide is at {}", slide_path.display());

            // safe to unwrap because we just parsed the slide
            let local_images = slide.local_images.as_ref().unwrap();
            trace!("Slide has {} local images", local_images.len());
            if local_images.is_empty() {
                continue;
            }

            for img in local_images {
                let im_path = PathBuf::from(img);
                let img_filename = im_path
                    .file_name()
                    .with_context(|| {
                        format!("Could not obtain file name of {}", im_path.display())
                    })?
                    .to_str()
                    .with_context(|| format!("{} is not valid UTF-8", im_path.display()))?;

                debug!("Image filename is {}", img_filename);
                if !img.starts_with("../img") {
                    // todo: this might not work on windows
                    warn!("This local image is not in the img/ directory (it's in `{}`) and will be skipped.", img);
                    continue;
                }
                let mut img_containing_dir = im_path.strip_prefix("..")?.to_path_buf();
                img_containing_dir.pop();

                let slide_dir = &slide_path.parent().with_context(|| {
                    format!(
                        "Could not get parent of slide at path `{}`",
                        slide_path.display()
                    )
                })?;
                let actual_img_path = fs::canonicalize(slide_dir.join(img))?;
                let img_dst_dir = output_dir.as_ref().join(img_containing_dir);
                let img_dst_path = img_dst_dir.join(img_filename);

                trace!("Attempting to create {}", img_dst_dir.display());
                fs::create_dir_all(&img_dst_dir)?;
                println!(
                    "Copying `{}` into `{}`",
                    actual_img_path.display(),
                    img_dst_path.display()
                );
                fs::copy(actual_img_path, img_dst_path)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_render() {
        let slides = vec![
            Slide::new("slide 1".to_string()),
            Slide::new("slide 2".to_string()),
            Slide::new("slide 3".to_string()),
        ];
        let ppt = Presentation {
            title: "Test Presentation".to_string(),
            template: "{{ slide_title }} {%for fc in ingested_files %}'{{fc}}'{%endfor%}"
                .to_string(),
            slides,
        };
        let render_result = ppt.render().unwrap();
        assert_eq!(
            render_result,
            "Test Presentation 'slide 1''slide 2''slide 3'"
        );
    }
}
