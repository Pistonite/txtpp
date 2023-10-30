//! Wrapper around Path objects provided by the standard library
//!
//! In the program, we use many types of paths
//! - The path of the shell executable
//! - The working directory of the main process
//! - The path to the current file being processed relative to the working directory
//! - The output path specified by the `temp` directive which may be relative or absolute
//!
//! The wrapper make sure that paths are always in the correct context.
//! It also adds some convenience methods and helpers to convert to and from
//! the standard library types.

use crate::error::PathError;
use error_stack::{Report, Result};
use std::ffi::OsString;
use std::path::PathBuf;

mod abs_path;
pub use abs_path::*;
mod directory;
pub use directory::*;

pub trait TxtppPath: Sized {
    /// Get the path to the txtpp file corresponding to the current path.
    ///
    /// For a file foo.bar, the txtpp file could be:
    /// - foo.bar.txtpp, or
    /// - foo.txtpp.bar
    ///
    /// If the path is already a txtpp file, `None` will be returned.
    /// Otherwise, if a corresponding txtpp file exists, that will be returned
    fn get_txtpp_file(&self) -> Option<Self>;

    /// Check if the path matches a txtpp file format. The path can be non-existent and this will
    /// still return true as long as the format is correct.
    fn is_txtpp_file(&self) -> bool;

    /// If current path is a txtpp file, return the path corresponding to the output of the txtpp
    /// file. This is the reverse of [`TxtppPath::get_txtpp_file`].
    ///
    /// If the path is not a txtpp file, an error will be returned.
    fn remove_txtpp(&self) -> Result<Self, PathError>;
}

impl TxtppPath for PathBuf {
    fn is_txtpp_file(&self) -> bool {
        match self.extension() {
            Some(ext) => {
                if ext == TXTPP_EXT {
                    return true;
                }
                // check if the second extension is txtpp
                let mut p = self.clone();
                p.set_extension("");
                match p.extension() {
                    Some(ext) => ext == TXTPP_EXT,
                    None => false,
                }
            }
            None => false,
        }
    }

    fn get_txtpp_file(&self) -> Option<Self> {
        if self.is_txtpp_file() {
            return None;
        }

        let mut p = self.clone();
        match self.extension() {
            Some(ext) => {
                // if there is extension, try .xxx.txtpp then .txtpp.xxx
                let mut ext1 = ext.to_os_string();
                ext1.push(".");
                ext1.push(TXTPP_EXT);
                p.set_extension(ext1);

                if p.is_file() {
                    return Some(p);
                }
                p.set_extension(""); // restore p
                let mut ext2 = OsString::from(TXTPP_EXT);
                ext2.push(".");
                ext2.push(ext);
                p.set_extension(ext2);
                if p.is_file() {
                    Some(p)
                } else {
                    None
                }
            }
            None => {
                // if there's no extension, the only way is adding .txtpp at the end
                let ext = OsString::from(TXTPP_EXT);
                p.set_extension(ext);
                if p.is_file() {
                    Some(p)
                } else {
                    None
                }
            }
        }
    }

    fn remove_txtpp(&self) -> Result<Self, PathError> {
        if !self.is_txtpp_file() {
            return Err(Report::new(PathError::from(self))
                .attach_printable(format!("path does not have {TXTPP_EXT} extension")));
        }
        let mut p = self.clone();
        p.set_extension("");
        if matches!(p.extension(), Some(ext) if ext == TXTPP_EXT) {
            // remove the txtpp extension and add the original back
            p.set_extension("");
            let self_ext = self.extension().ok_or_else(|| {
                Report::new(PathError::from(self))
                    .attach_printable(format!("path does not have {TXTPP_EXT} extension"))
            })?;
            p.set_extension(self_ext);
        }

        Ok(p)
    }
}

#[cfg(windows)]
#[inline]
pub fn normalize_path(p: &str) -> &str {
    if let Some(stripped) = p.strip_prefix(r"\\?\") {
        stripped
    } else {
        p
    }
}

#[cfg(not(windows))]
#[inline]
pub fn normalize_path(p: &str) -> &str {
    p
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_txtpp_file() {
        assert!(PathBuf::from("foo.txtpp").is_txtpp_file());
        assert!(PathBuf::from("foo.bar.txtpp").is_txtpp_file());
        assert!(PathBuf::from("foo.txtpp.bar").is_txtpp_file());
        assert!(!PathBuf::from("foo.bar").is_txtpp_file());
        assert!(!PathBuf::from("foo").is_txtpp_file());
        assert!(!PathBuf::from("txtpp").is_txtpp_file());
    }

    #[test]
    fn test_remove_txtpp() {
        assert!(PathBuf::from("foo").remove_txtpp().is_err());
        assert!(PathBuf::from("foo.bar").remove_txtpp().is_err());
        assert_eq!(
            PathBuf::from("foo.bar.txtpp").remove_txtpp().unwrap(),
            PathBuf::from("foo.bar")
        );
        assert_eq!(
            PathBuf::from("foo.txtpp.bar").remove_txtpp().unwrap(),
            PathBuf::from("foo.bar")
        );
    }
}
