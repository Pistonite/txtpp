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

mod abs_path;
use std::ffi::OsString;
use std::path::PathBuf;

pub use abs_path::*;
mod directory;
pub use directory::*;
mod error;
pub use error::*;

pub trait TxtppPath: Sized {
    /// Get the path with the txtpp extension,
    ///
    /// Return `.txtpp` added to self if:
    /// - self does not end with `.txtpp`
    /// - `{self}.txtpp` is a file
    /// Otherwise, returns None.
    fn get_txtpp_file(&self) -> Option<Self>;
    /// Check if the path has the txtpp extension itself
    fn is_txtpp_file(&self) -> bool;
}

impl TxtppPath for PathBuf {
    fn is_txtpp_file(&self) -> bool {
        match self.extension() {
            Some(ext) => ext == TXTPP_EXT,
            None => false,
        }
    }

    fn get_txtpp_file(&self) -> Option<Self> {
        if self.is_txtpp_file() {
            return None;
        }

        let mut p = self.clone();
        let ext = match p.extension() {
            Some(ext) => {
                let mut ext = ext.to_os_string();
                ext.push(".");
                ext.push(TXTPP_EXT);
                ext
            }
            None => OsString::from(TXTPP_EXT),
        };
        p.set_extension(ext);
        

        if p.is_file() {
            Some(p)
        } else {
            None
        }
    }
}
