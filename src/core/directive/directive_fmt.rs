use std::fmt::Write;
use error_stack::{IntoReport, Result};
use super::*;

impl Directive {
    pub fn format_output(whitespaces: &str, raw_output: &str, line_ending: &str) -> Result<String, DirectiveError> {
        let mut output = String::new();
        for line in raw_output.lines() {
            write!(output, "{whitespaces}{line}{line_ending}").into_report().map_err(|e|{
                e.change_context(DirectiveError).attach_printable("could not format output")
            })?;
        }
        Ok(output)
    }
}