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
pub use abs_path::*;
mod directory;
pub use directory::*;
mod txtpp_path;
pub use txtpp_path::*;
