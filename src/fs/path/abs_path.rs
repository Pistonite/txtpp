use crate::error::PathError;
use crate::fs::normalize_path;
use derivative::Derivative;
use error_stack::{IntoReport, Report, Result};
use std::fs;
use std::path::{Path, PathBuf};

use super::TxtppPath;

pub const TXTPP_EXT: &str = "txtpp";

/// Representation of an absolute path that exists.
///
/// Using [`PathBuf`] directly in the program can be confusing,
/// since it can represent both relative and absolute paths in different contexts.
/// Hense, we use `AbsPath` wherever we can to indicate that a path is resolved and absolute.
///
/// We still use [`PathBuf`] in places that usually represent input from the user,
/// as it could be relative or absolute and may not exist.
#[derive(Debug, Clone, Derivative)]
#[derivative(PartialEq, Eq, Hash)]
pub struct AbsPath {
    /// Base
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    b: PathBuf,
    /// Absolute path
    p: PathBuf,
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
    /// Directly contruct from a [`PathBuf`]. Used ONLY in unit tests
    pub fn new(p: PathBuf) -> Self {
        Self { b: p.clone(), p }
    }

    /// Convert a [`PathBuf`] to an absolute path as a base
    ///
    /// This will error if:
    /// - the path doesn't exist
    /// - the path cannot be made absolute for some reason
    ///
    /// If the path is relative, it will be made absolute by
    /// using [`canonicalize`](std::path::Path::canonicalize)
    pub fn create_base(p: PathBuf) -> Result<Self, PathError> {
        let p_abs = Self::make_abs(p)?;
        Ok(Self {
            b: p_abs.clone(),
            p: p_abs,
        })
    }

    /// Convert a [`PathBuf`] to an absolute path with the same base
    ///
    /// This will error if:
    /// - the path doesn't exist
    /// - the path cannot be made absolute for some reason
    ///
    /// If the path is relative, it will be made absolute by
    /// using [`canonicalize`](std::path::Path::canonicalize)
    pub fn share_base(&self, p: PathBuf) -> Result<Self, PathError> {
        Ok(Self {
            b: self.b.clone(),
            p: Self::make_abs(p)?,
        })
    }

    fn make_abs(p: PathBuf) -> Result<PathBuf, PathError> {
        if !p.exists() {
            return Err(Report::new(PathError::from(&p)).attach_printable("path does not exist"));
        }
        p.canonicalize().into_report().map_err(|e| {
            e.change_context(PathError::from(&p))
                .attach_printable("cannot resolve path as absolute")
        })
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
        let path_abs = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.p.join(path)
        };
        if !path_abs.exists() && create {
            create_file(&path_abs)?;
        }
        self.share_base(path_abs)
    }

    /// Get the parent
    pub fn parent(&self) -> Result<Self, PathError> {
        let p_parent_abs = match self.p.parent() {
            Some(p) => p,
            None => {
                return Err(Report::new(PathError::from(self))
                    .attach_printable("cannot get parent directory"))
            }
        };

        self.share_base(p_parent_abs.to_path_buf())
    }

    pub fn trim_txtpp(&self) -> Result<String, PathError> {
        let p = self.p.remove_txtpp()?;
        Ok(path_string_from_base(&self.b, &p))
    }
}

impl std::fmt::Display for AbsPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", path_string_from_base(&self.b, &self.p))
    }
}

fn create_file<P>(p: &P) -> Result<(), PathError>
where
    P: AsRef<Path>,
{
    log::debug!("creating file: {}", p.as_ref().display());
    fs::File::create(p).into_report().map_err(|e| {
        e.change_context(PathError::from(p))
            .attach_printable("cannot create file")
    })?;
    Ok(())
}

fn path_string_from_base<P>(base: &P, path: &P) -> String
where
    P: AsRef<Path>,
{
    let base = base.as_ref();
    let path = path.as_ref();
    let path = if base == path {
        path
    } else {
        match path.strip_prefix(base) {
            Ok(p) => p,
            Err(_) => path,
        }
    };
    normalize_path(&path.display().to_string()).to_string()
}
