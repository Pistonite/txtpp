use std::error;
use std::fmt;

#[derive(Debug)]
pub struct ExecuteError;

impl fmt::Display for ExecuteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Txtpp was unsuccessful!")
    }
}

impl error::Error for ExecuteError {}