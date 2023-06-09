//! Wrapper to perform file system operations
//!

mod line_ending;
pub use line_ending::GetLineEnding;
mod path;
pub use path::*;

mod shell;
pub use shell::{Shell, TXTPP_FILE};

mod io_context;
pub use io_context::*;
