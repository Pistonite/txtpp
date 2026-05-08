use crate::fs::normalize_path;
use derivative::Derivative;
use std::fs;
use std::path::{Path, PathBuf};

use cu::pre::*;

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
    pub fn create_base(p: PathBuf) -> cu::Result<Self> {
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
    pub fn share_base(&self, p: PathBuf) -> cu::Result<Self> {
        Ok(Self {
            b: self.b.clone(),
            p: Self::make_abs(p)?,
        })
    }

    fn make_abs(p: PathBuf) -> cu::Result<PathBuf> {
        // TODO: --rewrite: this should just be p.normalize_exists()
        if !p.exists() {
            cu::bail!(
                "path does not exist: '{}'",
                normalize_path(&p.display().to_string())
            );
        }
        match p.canonicalize() {
            Ok(p) => Ok(p),
            Err(e) => {
                cu::rethrow!(
                    e,
                    "connot convert path to absolute: '{}'",
                    normalize_path(&p.display().to_string())
                );
            }
        }
    }

    /// Resolve a path relative to the current path
    ///
    /// if `ext` is absolute, return `ext`, otherwise join `ext` with the current path.
    ///
    /// If create is `true`, the path will be created if it doesn't exist. Otherwise the path must exist.
    pub fn try_resolve<P>(&self, ext: &P, create: bool) -> cu::Result<Self>
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
    pub fn parent(&self) -> cu::Result<Self> {
        let p_parent_abs = self.p.parent_abs()?;
        self.share_base(p_parent_abs)
    }

    pub fn trim_txtpp(&self) -> cu::Result<String> {
        let p = self.p.remove_txtpp()?;
        Ok(path_string_from_base(&self.b, &p))
    }
}

impl std::fmt::Display for AbsPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", path_string_from_base(&self.b, &self.p))
    }
}

fn create_file<P>(p: &P) -> cu::Result<()>
where
    P: AsRef<Path>,
{
    // TODO: --rewrite: this should just be cu::fs::write with empty content?
    cu::debug!("creating file: {}", p.as_ref().display());
    cu::check!(
        fs::File::create(p),
        "cannot create file: '{}'",
        normalize_path(&p.as_ref().to_string_lossy())
    )?;
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
