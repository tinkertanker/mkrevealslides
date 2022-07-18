use crate::errors::ArgumentError;
use io::find_slides;
use crate::presentation::slide::SlideFile;
use crate::ui::cli::{CliArgs, Commands};
use crate::ui::conf::PresentationConfigFile;


use std::path::PathBuf;
use std::{env, fs};
use tera::Tera;
use tracing::{debug, trace, warn};

/// Utilities to work with Slides
pub mod slide;
/// Functions that work with the disk
pub mod io;

/// The logical representation of a presentation configuration
#[derive(Debug, Clone)]
pub struct PresentationConfig {
    /// Title of the presentation
    pub title: String,
    /// Output directory of the presentation
    /// Needs to exist
    // todo: support directories that don't yet exist
    pub output_directory: PathBuf,
    /// Output filename of the final presentation file, with extension
    pub output_filename: PathBuf,
    /// Absolute path to the template file
    pub template_file: PathBuf,
    /// Slides to be included in the presentation
    /// in the order that they appear in
    pub slides: Vec<SlideFile>,
}

impl PresentationConfig {
    /// Attempts to validate this PresentationConfig
    /// In particular, it checks that any paths
    /// specified are valid, and those that need to be
    /// accessed can be accessed.
    fn validate(&self) -> Result<(), ArgumentError> {
        trace!("Validating PresentationConfig");
        trace!("Checking output_file");
        // todo:

        let output_file = self.output_directory.join(&self.output_filename);

        // does it exist and is it a file?
        if output_file.is_file() {
            // if it exists, we will warn about overwriting it
            warn!(
                "Output file at `{}` already exists, will overwrite",
                output_file.display()
            );
        }
        trace!("Checking template_file");
        if !self.template_file.is_absolute() {
            return Err(ArgumentError::new(
                "template_file".to_string(),
                self.template_file.to_str().unwrap_or("<invalid path>"),
                "Template file must be an absolute path".to_string(),
            ));
        }

        if !self.template_file.is_file() {
            return Err(ArgumentError::new(
                "template_file".to_string(),
                self.template_file.to_str().unwrap_or("<invalid path>"),
                "Template file does not exist or cannot be read".to_string(),
            ));
        }
        Ok(())
    }

    /// Renders the presentation into a string
    ///
    /// # Returns
    /// Returns the contents of the presentation as a String
    ///
    /// # Errors
    /// If the template engine fails to render the presentation.
    fn render(&self) -> Result<String, tera::Error> {
        let mut ctx = tera::Context::new();
        let template = fs::read_to_string(&self.template_file)?;

        let slide_contents = self
            .slides
            .iter()
            .map(| s| &s.contents)
            .collect::<Vec<&String>>();
        ctx.insert("slide_title", &self.title);
        ctx.insert("ingested_files", &slide_contents);

        let result = Tera::one_off(&template, &ctx, false);
        trace!("Render template succeeded: {}", result.is_ok());
        result
    }

    /// Packages the presentation to a file.
    /// This will copy all local images referenced in slides into the output directory
    ///
    /// Optionally, downloads revealJS libs and generates the zip too
    pub fn package(&self) -> Result<(), anyhow::Error> {
        let output = self.render()?;
        debug!("Rendered {} bytes", output.len());
        let output_path = self.output_directory.join(&self.output_filename);
        debug!("Writing to `{}`", output_path.display());
        fs::write(&output_path, output)?;
        println!("Slides written to `{}`", output_path.display());

        for _slide in &self.slides {
            todo!()
        }
        Ok(())
    }
}

/// Attempts to convert CLI user input to PresentationConfig
/// All paths will be converted to absolute paths with respect to the current working directory.
/// (i.e. the directory the command was executed in)
impl TryFrom<CliArgs> for PresentationConfig {
    type Error = anyhow::Error;

    fn try_from(args: CliArgs) -> Result<Self, Self::Error> {
        match args.command {
            Commands::FromConfig { config_path } => {
                let config = PresentationConfigFile::read_config_file(config_path)?;
                Ok(Self::try_from(config)?)
            }
            Commands::FromCli {
                title,
                slide_dir,
                template_file,
                output_dir,
                output_file,
            } => {
                trace!("Converting CLI args to PresentationConfig");
                let cwd = fs::canonicalize(env::current_dir()?)?;
                let slide_title = if let Some(title) = title {
                    title
                } else {
                    "Untitled Presentation".to_string()
                };
                let slides = find_slides(&cwd.join(slide_dir))?;
                let cfg = PresentationConfig {
                    title: slide_title,
                    output_directory: cwd.join(output_dir),
                    output_filename: output_file,
                    template_file: cwd.join(template_file),
                    slides,
                };
                cfg.validate()?;
                Ok(cfg)
            }
        }
    }
}

/// Attempts to convert a PresentationConfigFile to PresentationConfig
/// Validates and converts relative paths to absolute paths in the process
impl TryFrom<PresentationConfigFile> for PresentationConfig {
    type Error = anyhow::Error;

    fn try_from(config: PresentationConfigFile) -> Result<Self, Self::Error> {
        trace!("Attempting to convert PresentationConfigFile to PresentationConfig");
        let include_files_abs_paths = config
            .include_files
            .iter()
            .map(|relative_pth| {
                config
                    .working_directory
                    .join(&config.slide_dir)
                    .join(relative_pth)
            })
            .collect::<Vec<PathBuf>>();
        trace!(
            "Converted {} include_file paths to abs paths",
            include_files_abs_paths.len()
        );
        let slides = if include_files_abs_paths.is_empty() {
            // let's try to search for slides
            find_slides(&config.working_directory.join(config.slide_dir))?
        } else {
            let sf = include_files_abs_paths
                .iter()
                .map(SlideFile::read_and_parse)
                .collect::<Result<Vec<SlideFile>, anyhow::Error>>()?;
            sf
        };

        let cfg = PresentationConfig {
            title: config.title,
            output_directory: config.working_directory.join("output"), // todo: add config option to configure output dir
            template_file: config.working_directory.join(config.template_file),
            output_filename: config.output_file,
            slides,
        };
        cfg.validate()?;
        Ok(cfg)
    }
}
