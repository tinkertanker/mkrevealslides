use serde::{Deserialize};

#[derive(Debug, Deserialize)]
pub struct SlideConfig {
    pub title: String,
    pub slide_dir: String,
    pub template_dir: String,
    pub include_files: Option<Vec<String>>,
}