//! # txtpp
//! A simple-to-use general purpose preprocessor for text files.
//!
//! ```no_compile
//! cargo install txtpp
//! ```
//!
//! See the [README on GitHub](https://github.com/iTNTPiston/txtpp) for more information, like the directive syntax and CLI usage.
//!
//! ## As a library
//! This library gives you access to the full functionalities of txtpp. The API consists of:
//!
//! - [`Config`] object to configure txtpp. This is what the CLI uses under the hood.
//! - [`txtpp`] and [`Txtpp::run`] to consume the [`Config`] and run txtpp.
//! - [`Mode`] and [`Verbosity`] used in the config
//! - [`error`] module for explicit error handling
//!
//! ## Examples
//! ### Running out-of-box
//! This runs `txtpp` with the [default config](Config::default),
//! which should mostly be the same as running `txtpp` from the CLI without args. However,
//! the library default and the CLI default may fork in the future.
//!
//! ```rust
//! use txtpp::{txtpp, Config};
//!
//! // Use the default config
//! txtpp(Default::default()).unwrap();
//! ```
//!
//! ### Overriding default config
//! Running with a custom config is most useful if you need run it from a different working directory
//! ```no_run
//! use txtpp::{txtpp, Config, Mode, Verbosity};
//! use std::path::PathBuf;
//!
//! let mut config = Config::default();
//! config.base_dir = "path/to/working/directory".into();
//! txtpp(config).unwrap();
//! ```
//!
//! ### Custom config and error handling
//! You can also create a [`Config`] from scratch
//! ```no_run
//! use txtpp::{Txtpp, Config, Mode, Verbosity};
//!
//! let config = Config {
//!     base_dir: "example".into(),
//!     shell_cmd: "pwsh -Command".to_owned(),
//!     inputs: vec!["myfile".to_owned(), "mydir".to_owned()],
//!     recursive: true,
//!     num_threads: 16,
//!     mode: Mode::Verify,
//!     verbosity: Verbosity::Quiet,
//!     trailing_newline: false,
//! };
//!
//! if let Err(e) = Txtpp::run(config) {
//!     println!("Txtpp returned an error!");
//!     /*
//!        custom error handling...
//!     */
//! }
//! ```
//! Note that we used [`Txtpp::run`] instead of [`txtpp`] here.
//! [`txtpp`] is just a wrapper around [`Txtpp::run`] that also prints the error.
//!
//! The config above is equivalent to running the CLI with the following args from the `example` directory
//! ```no_compile
//! txtpp -s "pwsh -Command" -r -j 16 -q verify myfile mydir
//! ```
//!
mod core;
pub use crate::core::{txtpp, Config, Mode, Txtpp, Verbosity};
pub mod error;
mod fs;
