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
        match self.kind {
            PpErrorKind::OpenFile => write!(f, "Could not open file."),
            PpErrorKind::ReadFile => write!(f, "Could not read file."),
            PpErrorKind::WriteFile => write!(f, "Could not write file."),
            PpErrorKind::DeleteFile => write!(f, "Could not remove file."),
            PpErrorKind::VerifyOutput => write!(f, "Output file not verified."),
            PpErrorKind::Directive => write!(f, "Error executing directive"),
            PpErrorKind::Other => write!(f, "Internal error."),
        }
    }
}

impl error::Error for PpError {}

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
    #[cfg(windows)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // on windows, we try to remove the `\\?\` prefix returned
        // by `std::path::Path::display` to make the error message
        // more readable
        let path = if self.path.starts_with(r"\\?\") {
            &self.path[4..]
        } else {
            &self.path
        };
        Self::fmt_internal(path, f)
    }
    #[cfg(not(windows))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Self::fmt_internal(&self.path, f)
    }
}

impl error::Error for PathError {}

impl PathError {
    fn fmt_internal(p: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error in path `{p}`")
    }
}
