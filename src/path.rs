use std::path::{Path, PathBuf};

/// Represents a relative path to the working directory of the `txtpp` process
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BaseRelPath {
    p: String,
}

impl BaseRelPath {
    pub fn join_base(&self, base: &BasePath) -> BaseAbsPath {
        BaseAbsPath {
            p: base.p.join(&self.p),
        }
    }
}

impl AsRef<String> for BaseRelPath {
    fn as_ref(&self) -> &String {
        &self.p
    }
}

impl AsRef<Path> for BaseRelPath {
    fn as_ref(&self) -> &Path {
        self.p.as_ref()
    }
}

impl From<String> for BaseRelPath {
    fn from(p: String) -> Self {
        BaseRelPath { p }
    }
}

impl From<BaseRelPath> for String {
    fn from(p: BaseRelPath) -> Self {
        p.p
    }
}

/// Represents an absolute path
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BaseAbsPath {
    p: PathBuf,
}

impl From<BaseAbsPath> for PathBuf {
    fn from(p: BaseAbsPath) -> Self {
        p.p
    }
}

impl AsRef<Path> for BaseAbsPath {
    fn as_ref(&self) -> &Path {
        self.p.as_ref()
    }
}

/// Represents the working directory of the `txtpp` process as an absolute path
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BasePath {
    p: PathBuf,
}

impl AsRef<PathBuf> for BasePath {
    fn as_ref(&self) -> &PathBuf {
        &self.p
    }
}

impl From<BasePath> for PathBuf {
    fn from(p: BasePath) -> Self {
        p.p
    }
}