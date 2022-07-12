use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct ArgumentError {
    pub arg: String,
    pub value: String,
    pub reason: String,
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

impl Display for ArgumentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "ArgError [{arg}=>{val}]: {reason}", arg=self.arg, val=self.value, reason=self.reason)
    }
}

impl Error for ArgumentError {
    fn description(&self) -> &str {
        &self.reason
    }
}