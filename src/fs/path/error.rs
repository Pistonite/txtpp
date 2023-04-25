use std::error;
use std::fmt;
use std::path::Path;

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
