//! Wrapper to perform file system operations

use crate::error::PathError;
use crate::fs::normalize_path;
use error_stack::{Result, ResultExt};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub const CRLF: &str = "\r\n";
pub const LF: &str = "\n";
#[cfg(windows)]
pub const OS_LINE_ENDING: &str = CRLF;
#[cfg(not(windows))]
pub const OS_LINE_ENDING: &str = LF;

pub trait GetLineEnding {
    fn get_line_ending(&self) -> Result<&'static str, PathError>;
}

impl<P> GetLineEnding for P
where
    P: AsRef<Path>,
{
    fn get_line_ending(&self) -> Result<&'static str, PathError> {
        let mut buf = vec![];
        let len = File::open(self)
            .map(BufReader::new)
            .and_then(|mut r| r.read_until(b'\n', &mut buf))
            .change_context_lazy(|| PathError::from(self))
            .attach_printable_lazy(|| {
                format!(
                    "Failed to get line ending for file: {}",
                    normalize_path(&self.as_ref().display().to_string())
                )
            })?;
        Ok(get_line_ending_from_buf(&buf, len))
    }
}

fn get_line_ending_from_buf(buf: &[u8], len: usize) -> &'static str {
    match len {
        0 => OS_LINE_ENDING,
        1 => {
            if buf[0] == b'\n' {
                LF
            } else {
                OS_LINE_ENDING
            }
        }
        _ => {
            if buf[len - 1] == b'\n' {
                if buf[len - 2] == b'\r' {
                    CRLF
                } else {
                    LF
                }
            } else {
                OS_LINE_ENDING
            }
        }
    }
}

#[cfg(test)]
mod ut {
    use super::*;

    macro_rules! test_line_ending {
        ($expected:expr, $buf:literal) => {
            let buf = $buf.to_string();
            assert_eq!(
                $expected,
                get_line_ending_from_buf(buf.as_bytes(), buf.len())
            );
        };
    }

    #[test]
    fn test_empty() {
        test_line_ending!(OS_LINE_ENDING, "");
    }

    #[test]
    fn test_text_nonewline() {
        test_line_ending!(OS_LINE_ENDING, "something");
    }

    #[test]
    fn test_text_lf() {
        test_line_ending!(LF, "something\n");
    }

    #[test]
    fn test_lf() {
        test_line_ending!(LF, "\n");
    }

    #[test]
    fn test_text_crlf() {
        test_line_ending!(CRLF, "something\r\n");
    }

    #[test]
    fn test_crlf() {
        test_line_ending!(CRLF, "\r\n");
    }

    #[test]
    fn test_cr() {
        test_line_ending!(OS_LINE_ENDING, "\r");
    }
}
