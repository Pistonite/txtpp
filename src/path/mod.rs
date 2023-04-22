//! Wrappers for [`std::path::Path`] and [`std::path::PathBuf`]
//! 
//! In the program, we use many types of paths
//! - The path of the shell executable
//! - The working directory of the main process
//! - The path to the current file being processed relative to the working directory
//! - The output path specified by the `temp` directive which may be relative or absolute
//! 
//! The wrapper make sure that paths are always in the correct context.


use std::{error, path::PathBuf};

mod abs_path;
pub use abs_path::*;
mod rel_path;
pub use rel_path::*;
mod path_spec;
pub use path_spec::*;

/// Internal error related to path resolution
#[derive(Debug)]
pub struct Error {
    pub offender: String,
    pub reason: String
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error resolving path `{}`: {}", self.offender, self.reason)
    }
}

impl error::Error for Error {}

pub struct Directory {
    pub input_files: Vec<AbsPath>,
    pub directories: Vec<PathBuf>,
}

impl Directory {
    pub fn new() -> Self {
        Self {
            input_files: vec![],
            directories: vec![],
        }
    }

}