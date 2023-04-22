use std::{io, fmt::Write};
use super::*;

impl Directive {
    pub fn format_output(&self, raw_output: &str, line_ending: &str) -> io::Result<String> {
        let mut output = String::new();
        for line in raw_output.lines() {
            match write!(output, "{}{}{}", &self.whitespaces, line, line_ending) {
                Ok(_) => (),
                Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
            }
        }
        Ok(output)
    }
}