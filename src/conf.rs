use serde::{Deserialize};

#[derive(Debug, Deserialize)]
pub struct SlideConfig {
    pub title: Option<String>,
    pub include_files: Option<Vec<String>>
}