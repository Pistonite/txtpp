pub const TXTPP_EXT: &str = ".txtpp";
pub const TXTPP_HASH: &str = "TXTPP#";
pub const TXTPP_FILE: &str = "TXTPP_FILE";
pub const TXTPP_DIRECTIVE_INDEX: &str = "TXTPP_DIRECTIVE_INDEX";
pub const CRLF: &'static str = "\r\n";
pub const LF: &'static str = "\n";
#[cfg(windows)]
pub const OS_LINE_ENDING: &'static str = "\r\n";
#[cfg(not(windows))]
pub const OS_LINE_ENDING: &'static str = "\n";