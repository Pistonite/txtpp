use super::AbsPath;

/// Represents a scanned directory, with txtpp input files and subdirectories.
#[derive(Debug)]
pub struct Directory {
    /// txtpp input files
    pub files: Vec<AbsPath>,
    /// subdirectories
    pub subdirs: Vec<AbsPath>,
}

impl Directory {
    pub fn new() -> Self {
        Self {
            files: vec![],
            subdirs: vec![],
        }
    }

}