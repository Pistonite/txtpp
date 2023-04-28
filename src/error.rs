//! Error types

use std::error;
use std::fmt;
use std::path::Path;

/// Top level error
#[derive(Debug)]
pub struct TxtppError;

impl fmt::Display for TxtppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Txtpp was unsuccessful! There might be error traces below."
        )
    }
}

impl error::Error for TxtppError {}

/// Preprocessor error
#[derive(Debug)]
pub struct PpError {
    pub kind: PpErrorKind,
    pub file: String,
    pub line: usize,
}
#[derive(Debug)]
pub enum PpErrorKind {
    OpenFile,
    ReadFile,
    WriteFile,
    DeleteFile,
    VerifyOutput,
    Directive,
    Other,
}

impl fmt::Display for PpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let source = format!("at `{}`:{}", self.file, self.line);
        match self.kind {
            PpErrorKind::OpenFile => write!(f, "Could not open file {source}"),
            PpErrorKind::ReadFile => write!(f, "Could not read file {source}"),
            PpErrorKind::WriteFile => write!(f, "Could not write file {source}"),
            PpErrorKind::DeleteFile => write!(f, "Could not remove file {source}"),
            PpErrorKind::VerifyOutput => write!(f, "Output file not verified {source}"),
            PpErrorKind::Directive => write!(f, "Error executing directive {source}"),
            PpErrorKind::Other => write!(f, "Internal error {source}"),
        }
    }
}

impl error::Error for PpError {}

use crate::fs::normalize_path;

/// Error related to paths
#[derive(Debug)]
pub struct PathError {
    /// The path that caused the error, as a string
    pub path: String,
}

impl<P> From<&P> for PathError
where
    P: AsRef<Path>,
{
    fn from(p: &P) -> Self {
        Self {
            path: p.as_ref().display().to_string(),
        }
    }
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error in path `{}`", normalize_path(&self.path))
    }
}

impl error::Error for PathError {}
