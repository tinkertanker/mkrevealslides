use crate::error_handling::AppError;
use crate::slide::Slide;
use crate::ui::PresentationConfig;
use std::fs;
use tera::{Context, Tera};
use tracing::trace;

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
    type Error = std::io::Error;

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
        trace!("Presentation title: {}", config.title);
        trace!(
            "Reading template_file at `{}`",
            config.output_file.display()
        );
        let template = fs::read_to_string(&config.template_file)?;
        trace!("Template file read: {} bytes", template.len());
        trace!("Reading {} slides", config.include_files.len());
        let slides = {
            let mut slides = Vec::new();
            for pth in &config.include_files {
                trace!("Reading slide at `{}`", pth.display());
                let slide = Slide::from_file(pth)?;
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
        let mut ctx = Context::new();

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

    /// Packages the presentation to a file
    ///
    /// Optionally, copies any local images to the destination directory
    /// Optionally, downloads revealJS libs and generates the zip too
    pub fn package(&self) -> Result<(), AppError> {
        todo!()
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
