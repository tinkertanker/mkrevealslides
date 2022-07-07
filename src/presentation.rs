use crate::conf::PresentationConfig;
use crate::error_handling::AppError;
use crate::slide::Slide;
use tera::{Context, Tera};
use tracing::{debug, trace};

pub struct Presentation {
    pub title: String,
    /// Contains the contents of the template to use for the presentation
    pub template: String,
    pub slides: Vec<Slide>,
}

impl Presentation {
    /// Creates a new Presentation object from a PresentationConfig
    /// Upon construction, this will read the template file's contents,
    /// and all the contents of the slides.
    ///
    /// # Errors
    /// Returns an error if the template file could not be read
    /// Returns an error if the slides could not be read
    pub fn from_config(config: &PresentationConfig) -> Result<Self, AppError> {
        let template = std::fs::read_to_string(&config.template_file)?;
        debug!("Template read: {} bytes", template.len());
        let slides = {
            let mut slides = Vec::new();
            for pth in &config.to_full_paths()? {
                trace!("Reading slide at {}", pth.display());
                let slide = Slide::from_file(pth)?;
                slides.push(slide);
            }
            slides
        };
        debug!("Read {} slides", slides.len());
        Ok(Self {
            title: config.title.clone(),
            template,
            slides,
        })
    }

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

    // #[test]
    // fn test_from_config() {
    //     let cfg = PresentationConfig {
    //         title: "Test Presentation".to_string(),
    //         slide_dir: Default::default(),
    //         output_file: Default::default(),
    //         template_file: "test_template.html".to_string(),
    //         slides: vec![
    //             "test_slide1.md".to_string(),
    //             "test_slide2.md".to_string(),
    //             "test_slide3.md".to_string()
    //         ],
    //         include_files: None
    //     };
    // }
}
