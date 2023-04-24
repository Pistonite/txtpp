use termcolor::{Color, ColorSpec, StandardStream, WriteColor, ColorChoice};
use std::error::Error;
use std::io::Write;
use std::time::{Duration, Instant};

/// Utility for displaying progress
pub struct Progress {
    out: StandardStream,
    done_count: usize,
    total_count: usize,
    last_update: Instant,
}
impl Progress {
    pub fn new() -> Self {
        Self {
            out: StandardStream::stderr(ColorChoice::Always),
            done_count: 0,
            total_count: 0,
            last_update: Instant::now(),
        }
    }
    /// Update the progress display
    pub fn update_progress(&mut self, done: usize, total: usize) -> Result<(), Box<dyn Error>> {
        self.done_count = done;
        self.total_count = total;
        if self.last_update.elapsed() > Duration::from_millis(100) {
            self.update_progress_internal()?;
        }

        Ok(())
    }

    fn update_progress_internal(&mut self) -> Result<(), Box<dyn Error>> {
        self.out.set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Yellow)))?;
        let progress = format!("{}/{}", self.done_count, self.total_count);
        write!(self.out, "{:>12}", crate::core::verbs::SCANNED)?;
        self.out.reset()?;
        write!(self.out, " {}\r", progress)?;
        self.last_update = Instant::now();

        Ok(())
    }

    // pub fn print_progress(&mut self, done: usize, total: usize) -> Result<(), Box<dyn Error>> {
    //     let progress = format!("{}/{}", done, total);
    //     write!(self.stderr, "{:>12}\r", progress)?;
        
    //     Ok(())
    // }

    pub fn print_status(&mut self, status: &str, message: &str, color: Color, verbose: bool) -> Result<(), Box<dyn Error>> {
        self.out.reset()?;
        self.out.set_color(ColorSpec::new().set_bold(true).set_fg(Some(color)))?;
        write!(self.out, "{:>12}", status)?;
        self.out.reset()?;
        writeln!(self.out, " {}", message)?;
        if self.done_count < self.total_count {
            self.update_progress_internal()?;
        }
        Ok(())
    }
}