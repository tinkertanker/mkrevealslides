use core::fmt;
use std::error::Error;
use std::io;
use std::num::ParseIntError;

pub struct ArgumentError {
    pub arg: String,
    pub value: String,
    pub reason: String,
}

pub struct ValidationError {
    pub value: String,
    pub reason: String
}

impl ValidationError {
    pub fn new(value: &str, reason: String) -> Self {
        Self { value: value.to_string(), reason }
    }
}

impl ArgumentError {
    pub fn new(arg: String, value: &str, reason: String) -> Self {
        ArgumentError {
            arg,
            value: value.to_string(),
            reason,
        }
    }
}

// todo: support better error kinds
pub struct AppError {
    pub error_kind: String,
    pub description: String,
}

impl fmt::Debug for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.error_kind, self.description)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.error_kind, self.description)
    }
}

impl AppError {
    pub fn new(message: &str) -> AppError {
        AppError {
            error_kind: "App Error".to_string(),
            description: message.to_string(),
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
            error_kind: "IO Error".to_string(),
            description: err.to_string(),
        }
    }
}

impl From<ParseIntError> for AppError {
    fn from(err: ParseIntError) -> Self {
        Self {
            error_kind: "Parse Error (int)".to_string(),
            description: err.to_string(),
        }
    }
}

impl From<serde_yaml::Error> for AppError {
    fn from(err: serde_yaml::Error) -> Self {
        Self {
            error_kind: "Parse Error (yaml)".to_string(),
            description: err.to_string(),
        }
    }
}

impl From<tera::Error> for AppError {
    fn from(err: tera::Error) -> Self {
        Self {
            error_kind: "Template Engine Error".to_string(),
            description: err.to_string(),
        }
    }
}

impl From<ArgumentError> for AppError {
    fn from(err: ArgumentError) -> Self {
        Self {
            error_kind: format!("Arg error [{}=>{}]", err.arg, err.value),
            description: err.reason,
        }
    }
}

impl From<ValidationError> for AppError {
    fn from(err: ValidationError) -> Self {
        Self {
            error_kind: format!("Validation error [=>{}]", err.value),
            description: err.reason,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::BTreeMap;
    #[test]
    fn test_apperror_fmt_debug() {
        let err = AppError::new("test");
        assert_eq!(format!("{:?}", err), "App Error: test");
    }

    #[test]
    fn test_apperror_display() {
        let err = AppError::new("test");
        assert_eq!(format!("{}", err), "App Error: test");
    }

    #[test]
    fn test_apperror_description() {
        let err = AppError::new("test_desc");
        assert_eq!(err.to_string(), "App Error: test_desc");
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
        let bad_yaml_err =
            serde_yaml::from_str::<BTreeMap<String, String>>("invalid_yaml|").unwrap_err();
        let err = AppError::from(bad_yaml_err);
        assert!(err.to_string().contains("Parse Error (yaml)"));
    }

    #[test]
    fn test_from_argument_error() {
        let err = AppError::from(ArgumentError {
            arg: "arg0".to_string(),
            value: "test_value".to_string(),
            reason: "test_reason".to_string(),
        });
        assert_eq!(err.to_string(), "Arg error [arg0=>test_value]: test_reason");
    }

    #[test]
    fn test_from_tera_error() {
        let err = tera::Error::msg("welp".to_string());
        let err = AppError::from(err);
        assert_eq!(err.to_string(), "Template Engine Error: welp");
    }
}
