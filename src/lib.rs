//! # txtpp
//! A simple-to-use general purpose preprocessor for text files.
//! See the [README on GitHub](https://github.com/iTNTPiston/txtpp) for more information.
//!
//! # txtpp as a library
//! TODO
//!

mod core;
pub use crate::core::{txtpp, Config, Mode, Txtpp, Verbosity};
pub mod error;
mod fs;

// pub fn build() -> Result<(), PreprocessError> {

// }

// pub fn verify() -> Result<(), PreprocessError> {

// }

// pub fn clean() -> Result<(), PreprocessError> {

// }
