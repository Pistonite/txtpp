//! Error types

use std::error;
use std::fmt;

/// Preprocessor error
#[derive(Debug)]
pub struct PpError {
    pub kind: PpErrorKind,
    pub file: String,
    pub line: usize,
}
#[derive(Debug, Clone, Copy)]
pub enum PpErrorKind {
    OpenFile,
    ReadFile,
    WriteFile,
    DeleteFile,
    VerifyOutput,
    Directive,
    Other,
}
impl PpErrorKind {
    pub fn message(self) -> &'static str {
        match self {
            PpErrorKind::OpenFile => "could not open file",
            PpErrorKind::ReadFile => "could not read file",
            PpErrorKind::WriteFile => "could not write file",
            PpErrorKind::DeleteFile => "could not remove file",
            PpErrorKind::VerifyOutput => "failed to verify output file",
            PpErrorKind::Directive => "error executing directive",
            PpErrorKind::Other => "internal error",
        }
    }
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
