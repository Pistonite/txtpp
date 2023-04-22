use super::{Error, PathSpec, RelPath};
use std::path::{Path, PathBuf};

pub const TXTPP_EXT: &str = ".txtpp";

/// The path that can represent:
/// - The path of the shell executable
/// - The working directory of the main process
///
/// The path is always absolute
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AbsPath {
    p: PathBuf,
}

impl TryFrom<PathBuf> for AbsPath {
    type Error = Error;

    fn try_from(p: PathBuf) -> Result<Self, Self::Error> {
        if p.is_absolute() {
            Ok(AbsPath { p })
        } else {
            Err(Error {
                offender: p.display().to_string(),
                reason: "Path is not absolute".to_string(),
            })
        }
    }
}

impl From<AbsPath> for PathBuf {
    fn from(p: AbsPath) -> Self {
        p.p
    }
}

impl AbsPath {
    // /// Join with a relative path
    // pub fn join(&self, ext: &RelPath) -> Self {
    //     self.join_internal(ext)
    // }

    /// Resolve a path relative to the current path
    ///
    /// if `ext` is absolute, return `ext`, otherwise join `ext` with the current path
    pub fn resolve(&self, ext: &PathSpec) -> Self {
        let path: &Path = ext.as_ref();
        if path.is_absolute() {
            AbsPath {
                p: path.to_path_buf(),
            }
        } else {
            self.join_internal(ext)
        }
    }

    fn join_internal<P>(&self, ext: &P) -> Self
    where
        P: AsRef<Path>,
    {
        AbsPath {
            p: self.p.join(ext),
        }
    }

    /// Get the parent
    pub fn parent(&self) -> Result<Self, Error> {
        self.p
            .parent()
            .map(|p| AbsPath { p: p.to_path_buf() })
            .ok_or_else(|| Error {
                offender: self.p.display().to_string(),
                reason: "Cannot get parent directory.".to_string(),
            })
    }

    /// Check if the path ends with txtpp extension
    pub fn is_txtpp_file(&self) -> bool {
        self.p.ends_with(TXTPP_EXT)
    }

    /// Remove the txtpp extension
    pub fn trim_txtpp(&self) -> Result<Self, Error> {
        if !self.is_txtpp_file() {
            return Err(Error {
                offender: self.p.display().to_string(),
                reason: format!("Path does not have {} extension", TXTPP_EXT),
            });
        }
        let mut p = self.p.clone();
        p.set_extension("");
        Ok(AbsPath { p })
    }

    /// Get the file name with the txtpp extension,
    ///
    /// Return `.txtpp` added to self if:
    /// - self does not end with `.txtpp`
    /// - `{self}.txtpp` is a file
    /// Otherwise, returns None.
    pub fn get_txtpp_file(&self) -> Option<Self> {
        if self.is_txtpp_file() {
            return None;
        }
        let mut p = self.p.display().to_string();
        p.push_str(TXTPP_EXT);
        let p = Path::new(&p).canonicalize();
        Some(Self {
            p: p.ok().filter(|p| p.is_file())?,
        })
    }
}

impl AsRef<Path> for AbsPath {
    fn as_ref(&self) -> &Path {
        self.p.as_ref()
    }
}

impl std::fmt::Display for AbsPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.p.display())
    }
}
