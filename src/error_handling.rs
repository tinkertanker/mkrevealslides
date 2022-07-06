use core::fmt;
use std::error::Error;
use std::io;
use std::num::ParseIntError;
use crate::conf::ArgumentError;
use crate::val::ValError;


pub struct AppError {
    pub message: String, // todo: message is a bit misleading
    pub description: String
}

impl fmt::Debug for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.message, self.description)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.message, self.description)
    }
}

impl AppError {
    pub fn new(message: &str) -> AppError {
        AppError {
            message: message.to_string(),
            description: message.to_string()
        }
    }
}

impl Error for AppError {
    fn description(&self) -> &str {
        &self.description
    }
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        Self {
            message: "IO Error".to_string(),
            description: err.to_string()
        }
    }
}

impl From<ParseIntError> for AppError {
    fn from(err: ParseIntError) -> Self {
        Self {
            message: "Parse Error (int)".to_string(),
            description: err.to_string()
        }
    }
}

impl From<serde_yaml::Error> for AppError {
    fn from(err: serde_yaml::Error) -> Self {
        Self {
            message: "Parse Error (yaml)".to_string(),
            description: err.to_string()
        }
    }
}

impl From<ValError> for AppError {
    fn from(err: ValError) -> Self {
        Self {
            message: "Validation Error".to_string(),
            description: err
        }
    }
}

impl From<ArgumentError> for AppError {
    fn from(err: ArgumentError) -> Self {
        Self {
            message: format!("Arg error: {}", err.0),
            description: err.1
        }
    }
}