use super::{PathError, TxtppPath};
use std::fs;
use std::path::{Path, PathBuf};

use error_stack::{IntoReport, Report, Result};

pub const TXTPP_EXT: &str = "txtpp";

/// Representation of an absolute path that exists.
///
/// Using [`PathBuf`] directly in the program can be confusing,
/// since it can represent both relative and absolute paths in different contexts.
/// Hense, we use `AbsPath` wherever we can to indicate that a path is resolved and absolute.
///
/// We still use [`PathBuf`] in places that usually represent input from the user,
/// as it could be relative or absolute and may not exist.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AbsPath {
    p: PathBuf,
}

impl TryFrom<PathBuf> for AbsPath {
    type Error = Report<PathError>;

    /// Convert a [`PathBuf`] to an absolute path.
    ///
    /// This will error if:
    /// - the path doesn't exist
    /// - the path cannot be made absolute for some reason
    ///
    /// If the path is relative, it will be made absolute by
    /// using [`canonicalize`](std::path::Path::canonicalize)
    fn try_from(p: PathBuf) -> Result<Self, PathError> {
        if !p.exists() {
            return Err(Report::new(PathError::from(&p)).attach_printable("path does not exist"));
        }
        let p_abs = p.canonicalize().into_report().map_err(|e| {
            e.change_context(PathError::from(&p)).attach_printable("cannot resolve path as absolute")
            
        })?;

        Ok(Self { p: p_abs })
    }
}

/// Integration with [`PathBuf`] and [`Path`]
impl AbsPath {
    #[inline]
    pub fn as_path_buf(&self) -> &PathBuf {
        &self.p
    }
    #[inline]
    pub fn into_path_buf(self) -> PathBuf {
        self.p
    }
    #[inline]
    pub fn as_path(&self) -> &Path {
        self.p.as_path()
    }
}

impl From<AbsPath> for PathBuf {
    #[inline]
    fn from(p: AbsPath) -> Self {
        p.p
    }
}

impl AsRef<PathBuf> for AbsPath {
    #[inline]
    fn as_ref(&self) -> &PathBuf {
        self.as_path_buf()
    }
}

impl AsRef<Path> for AbsPath {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl AbsPath {
    /// Directly contruct from a [`PathBuf`]. Used ONLY for unit tests
    pub fn new(p: PathBuf) -> Self
    {
        Self { p }
    }
    /// Resolve a path relative to the current path
    ///
    /// if `ext` is absolute, return `ext`, otherwise join `ext` with the current path.
    ///
    /// If create is `true`, the path will be created if it doesn't exist. Otherwise the path must exist.
    pub fn try_resolve<P>(&self, ext: &P, create: bool) -> Result<Self, PathError>
    where
        P: AsRef<Path>,
    {
        let path: &Path = ext.as_ref();
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.p.join(path)
        };
        if !path.exists() && create {
            create_file(&path)?;
        }
        Self::try_from(path)
    }

    /// Get the parent
    pub fn parent(&self) -> Result<Self, PathError> {
        let p_parent = match self.p.parent() {
            Some(p) => p,
            None => {
                return Err(Report::new(PathError::from(self)).attach_printable("cannot get parent directory"))
            }
        };
        Self::try_from(p_parent.to_path_buf())
    }

    /// Remove the txtpp extension and resolve the path.
    ///
    /// If the resulting path does not exist, a new file will be created if
    /// `create` is true, otherwise an error will be returned.
    ///
    /// If the path does not have the txtpp extension, an error will be returned.
    pub fn trim_txtpp(&self, create: bool) -> Result<Self, PathError> {
        if !self.p.is_txtpp_file() {
            return Err(Report::new(PathError::from(self)).attach_printable(format!("path does not {TXTPP_EXT} extension")));
        }
        let mut p = self.p.clone();
        p.set_extension("");
        
        if !p.exists() && create {
            create_file(&p)?;
        }
        Self::try_from(p)
    }

}

impl std::fmt::Display for AbsPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.p.display())
    }
}

fn create_file<P>(p: &P) -> Result<(), PathError> where P: AsRef<Path>{
    fs::File::create(&p).into_report().map_err(|e| {
        e.change_context(PathError::from(&p)).attach_printable("cannot create file")
    })?;
    Ok(())
}
