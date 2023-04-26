//! Wrapper to perform file system operations

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
    fn get_line_ending(&self) -> &'static str;
}

impl<P> GetLineEnding for P
where
    P: AsRef<Path>,
{
    fn get_line_ending(&self) -> &'static str {
        let mut buf = vec![];
        File::open(self)
            .map(BufReader::new)
            .and_then(|mut r| {
                r.read_until(b'\n', &mut buf)?;
                Ok(get_line_ending_from_buf(&buf))
            })
            .unwrap_or(OS_LINE_ENDING)
    }
}

fn get_line_ending_from_buf(buf: &[u8]) -> &'static str {
    match buf.len() {
        0 => OS_LINE_ENDING,
        1 => {
            if buf[0] == b'\n' {
                LF
            } else {
                OS_LINE_ENDING
            }
        }
        _ => {
            if buf[buf.len() - 1] == b'\n' {
                if buf[buf.len() - 2] == b'\r' {
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

    #[test]
    fn test_empty() {
        let buf = vec![];
        assert_eq!(OS_LINE_ENDING, get_line_ending_from_buf(&buf));
    }

    #[test]
    fn test_text_nonewline() {
        let buf = "something".to_string();
        assert_eq!(OS_LINE_ENDING, get_line_ending_from_buf(buf.as_bytes()));
    }

    #[test]
    fn test_text_lf() {
        let buf = "something\n".to_string();
        assert_eq!(LF, get_line_ending_from_buf(buf.as_bytes()));
    }

    #[test]
    fn test_lf() {
        let buf = "\n".to_string();
        assert_eq!(LF, get_line_ending_from_buf(buf.as_bytes()));
    }

    #[test]
    fn test_text_crlf() {
        let buf = "something\r\n".to_string();
        assert_eq!(CRLF, get_line_ending_from_buf(buf.as_bytes()));
    }

    #[test]
    fn test_crlf() {
        let buf = "\r\n".to_string();
        assert_eq!(CRLF, get_line_ending_from_buf(buf.as_bytes()));
    }

    #[test]
    fn test_cr() {
        let buf = "\r".to_string();
        assert_eq!(OS_LINE_ENDING, get_line_ending_from_buf(buf.as_bytes()));
    }
}
