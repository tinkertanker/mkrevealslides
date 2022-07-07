use core::fmt;
use std::error::Error;
use std::io;
use std::num::ParseIntError;
use crate::conf::ArgumentError;
use crate::val::ValError;

// todo: support error kinds
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

impl From<tera::Error> for AppError {
    fn from(err: tera::Error) -> Self {
        Self {
            message: "Template Engine Error".to_string(),
            description: err.to_string()
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;
    use super::*;
    #[test]
    fn test_apperror_fmt_debug() {
        let err = AppError::new("test");
        assert_eq!(format!("{:?}", err), "test: test");
    }

    #[test]
    fn test_apperror_display() {
        let err = AppError::new("test");
        assert_eq!(format!("{}", err), "test: test");
    }

    #[test]
    fn test_apperror_description() {
        let err = AppError::new("test_desc");
        assert_eq!(err.to_string(), "test_desc: test_desc");
    }

    #[test]
    fn test_from_io_error() {
        let err = AppError::from(io::Error::new(io::ErrorKind::Other, "test"));
        assert_eq!(err.to_string(), "IO Error: test");
    }

    #[test]
    fn test_from_parse_int_error() {
        let err = AppError::from("not_num".parse::<i32>().unwrap_err());
        assert!(err.to_string().contains("Parse Error (int)"));
    }

    #[test]
    fn test_from_serde_yaml_error() {
        let bad_yaml_err = serde_yaml::from_str::
        <BTreeMap<String, String>>("invalid_yaml|").unwrap_err();
        let err = AppError::from(bad_yaml_err);
        assert!(err.to_string().contains("Parse Error (yaml)"));
    }

    #[test]
    fn test_from_argument_error() {
        let err = AppError::from(("arg0".to_string(), "invalid arg".to_string()));
        assert_eq!(err.to_string(), "Arg error: arg0: invalid arg");
    }

    #[test]
    fn test_from_val_error() {
        let err = AppError::from("welp".to_string());
        assert_eq!(err.to_string(), "Validation Error: welp");
    }

    #[test]
    fn test_from_tera_error() {
        let err = tera::Error::msg("welp".to_string());
        let err = AppError::from(err);
        assert_eq!(err.to_string(), "Template Engine Error: welp");
    }

}