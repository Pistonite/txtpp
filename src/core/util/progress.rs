use crate::core::Verbosity;
use std::error::Error;
use std::io::Write;
use std::time::{Duration, Instant};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// Utility for displaying progress
#[derive(Debug)]
pub struct Progress {
    out: StandardStream,
    pub done_count: usize,
    pub total_count: usize,
    last_update: Instant,
    verbosity: Verbosity,
    pub has_error: bool,
}
impl Progress {
    pub fn new(verbosity: Verbosity) -> Self {
        Self {
            out: StandardStream::stderr(ColorChoice::Always),
            done_count: 0,
            total_count: 0,
            last_update: Instant::now(),
            verbosity,
            has_error: false,
        }
    }
    pub fn is_done(&self) -> bool {
        self.done_count == self.total_count
    }

    pub fn add_done(&mut self, count: usize) -> Result<(), Box<dyn Error>> {
        self.done_count += count;
        self.update_progress()
    }
    pub fn add_done_quiet(&mut self, count: usize) {
        self.done_count += count;
    }
    pub fn add_total(&mut self, count: usize) -> Result<(), Box<dyn Error>> {
        self.total_count += count;
        self.update_progress()
    }

    /// Update the progress display
    pub fn update_progress(&mut self) -> Result<(), Box<dyn Error>> {
        if self.verbosity == Verbosity::Quiet {
            return Ok(());
        }
        if self.last_update.elapsed() > Duration::from_millis(100) {
            self.update_progress_internal()?;
        }

        Ok(())
    }

    fn update_progress_internal(&mut self) -> Result<(), Box<dyn Error>> {
        self.out
            .set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Yellow)))?;
        let progress = format!("{}/{}", self.done_count, self.total_count);
        write!(self.out, "{:>12}", crate::core::verbs::SCANNED)?;
        self.out.reset()?;
        write!(self.out, " {}\r", progress)?;
        self.last_update = Instant::now();

        Ok(())
    }

    pub fn print_status(
        &mut self,
        status: &str,
        message: &str,
        color: Color,
        verbose: bool,
    ) -> Result<(), Box<dyn Error>> {
        if self.verbosity == Verbosity::Quiet {
            return Ok(());
        }
        if verbose && self.verbosity != Verbosity::Verbose {
            return Ok(());
        }
        self.out.reset()?;
        self.out
            .set_color(ColorSpec::new().set_bold(true).set_fg(Some(color)))?;
        write!(self.out, "{:>12}", status)?;
        self.out.reset()?;
        // write the message then cleared to the end of line
        writeln!(self.out, " {}{esc}[0K", message, esc = 27 as char)?;
        if self.done_count < self.total_count {
            self.update_progress_internal()?;
        }
        Ok(())
    }
}
