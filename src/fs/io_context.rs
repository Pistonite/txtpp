use crate::error::{PpError, PpErrorKind};
use crate::fs::{normalize_path, AbsPath, GetLineEnding, TxtppPath};
use crate::Mode;
use error_stack::{Report, Result, ResultExt};
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Lines, Read, Write};
use std::path::{Path, PathBuf};

/// Context for processing a txtpp file.
///
/// This is an IO wrapper for reading from txtpp file and writing to the output file.
#[derive(Debug)]
pub struct IOCtx {
    /// Input reader
    input: Lines<BufReader<File>>,
    /// Output wrapper
    out: CtxOut,
    pub cur_line: usize,
    pub work_dir: AbsPath,
    pub line_ending: &'static str,
    pub input_path: String,
}

impl IOCtx {
    /// Create a new IO context for the input txtpp file.
    pub fn new(input_file: &AbsPath, mode: Mode) -> Result<Self, PpError> {
        let input_path = input_file.to_string();

        let line_ending = input_file.get_line_ending().map_err(|e| {
            e.change_context(Self::make_error_with_kind(
                input_path.clone(),
                PpErrorKind::OpenFile,
            ))
            .attach_printable(format!(
                "could not read line ending for input file: `{input_path}`"
            ))
        })?;

        let r = File::open(input_file)
            .map(BufReader::new)
            .change_context_lazy(|| {
                Self::make_error_with_kind(input_path.clone(), PpErrorKind::OpenFile)
            })
            .attach_printable_lazy(|| format!("could not open input file: `{input_path}`"))?;

        let output_path = input_file.as_path_buf().remove_txtpp().map_err(|e| {
            e.change_context(Self::make_error_with_kind(
                input_path.clone(),
                PpErrorKind::OpenFile,
            ))
            .attach_printable(format!(
                "could not resolve output path for input file: `{input_path}`"
            ))
        })?;

        let out = CtxOut::new(mode, &input_path, &output_path)?;

        let work_dir = input_file.parent().map_err(|e| {
            e.change_context(Self::make_error_with_kind(
                input_path.clone(),
                PpErrorKind::OpenFile,
            ))
            .attach_printable(format!(
                "cannot get working directory for input file: {}",
                input_file
            ))
        })?;

        Ok(Self {
            input: r.lines(),
            out,
            work_dir,
            line_ending,
            input_path,
            cur_line: 0,
        })
    }

    /// Get the next line from the input file.
    pub fn next_line(&mut self) -> Option<Result<String, PpError>> {
        let line = self.input.next().map(|line| {
            line.change_context_lazy(|| make_error!(self, PpErrorKind::ReadFile))
                .attach_printable("cannot read next line")
        });
        if line.is_some() {
            self.cur_line += 1;
        }
        line
    }

    /// Write to output.
    ///
    /// The output string should already have line endings resolved. This function writes the
    /// output directly as is.
    pub fn write_output(&mut self, output: &str) -> Result<(), PpError> {
        match &mut self.out {
            CtxOut::Build { path, out } => out
                .write_all(output.as_bytes())
                .change_context_lazy(|| make_error!(self, PpErrorKind::WriteFile))
                .attach_printable_lazy(|| format!("cannot write to `{}`", path.display())),
            CtxOut::InMemoryBuild { out, .. } => {
                out.push_str(output);
                Ok(())
            }
            CtxOut::Clean { .. } => Ok(()), // do nothing
            CtxOut::Verify { path, out, rem } => {
                log::debug!("verifying content: {output:?}");
                // len is the length in bytes
                let len = output.len() as u64;
                if *rem < len {
                    log::debug!("not enough content to verify: need {len}, remaining {rem}");
                    return Err(make_verify_report!(self, path));
                }
                let mut buf = vec![0; output.len()];
                out.read_exact(&mut buf)
                    .change_context_lazy(|| make_error!(self, PpErrorKind::ReadFile))
                    .attach_printable("cannot read from output file.")?;
                if buf != output.as_bytes() {
                    let string = String::from_utf8_lossy(&buf);
                    log::debug!("content different, actual: {string:?}");
                    return Err(make_verify_report!(self, path));
                }
                *rem -= len;
                Ok(())
            }
        }
    }

    /// Write a temp file to the working directory.
    pub fn write_temp_file(&mut self, temp_path: &str, contents: &str) -> Result<(), PpError> {
        let p = PathBuf::from(temp_path);

        if let CtxOut::Clean { .. } = self.out {
            if let Ok(export_file) = self.work_dir.try_resolve(&p, false) {
                fs::remove_file(&export_file)
                    .change_context_lazy(|| make_error!(self, PpErrorKind::DeleteFile))
                    .attach_printable_lazy(|| {
                        format!("could not remove temp file: `{export_file}`")
                    })?;
            }
            return Ok(());
        }

        log::debug!("writing temp file: {}", p.display());
        let export_file = self.work_dir.try_resolve(&p, true).map_err(|e| {
            e.change_context(make_error!(self, PpErrorKind::WriteFile))
                .attach_printable(format!("could not resolve temp file: `{}`", p.display()))
        })?;
        if export_file.as_path().is_dir() {
            return Err(Report::new(make_error!(self, PpErrorKind::WriteFile))
                .attach_printable(format!("cannot write to directory: `{export_file}`")));
        }
        // Check if the temp file already exists and has the same content
        if export_file.as_path().exists() {
            let current_content = fs::read_to_string(&export_file)
                .change_context_lazy(|| make_error!(self, PpErrorKind::ReadFile))
                .attach_printable_lazy(|| {
                    format!("could not read existing temp file: `{export_file}`")
                })?; // early return because if we can't read it, we probably can't write it either
            if current_content == contents {
                log::debug!("temp file already exists with same content, skipping");
                return Ok(());
            }
        }

        fs::write(&export_file, contents)
            .change_context_lazy(|| make_error!(self, PpErrorKind::WriteFile))
            .attach_printable_lazy(|| format!("could not write temp file: `{export_file}`"))
    }

    /// Finish
    pub fn done(mut self) -> Result<(), PpError> {
        match &mut self.out {
            CtxOut::Build { path, out } => out
                .flush()
                .change_context_lazy(|| make_error!(self, PpErrorKind::WriteFile))
                .attach_printable_lazy(|| format!("could not write to `{}`", path.display())),
            CtxOut::InMemoryBuild { path, out } => {
                if path.as_path().exists() {
                    let current_content = fs::read_to_string(path.as_path())
                        .change_context_lazy(|| make_error!(self, PpErrorKind::ReadFile))
                        .attach_printable_lazy(|| {
                            format!("could not read existing output file: `{}`", path.display())
                        })?; // early return because if we can't read it, we probably can't write it either
                    if &current_content == out {
                        log::debug!("output file already exists with same content, skipping");
                        return Ok(());
                    }
                }
                fs::write(path.as_path(), out)
                    .change_context_lazy(|| make_error!(self, PpErrorKind::WriteFile))
                    .attach_printable_lazy(|| {
                        format!("could not write output file: `{}`", path.display())
                    })
            }
            CtxOut::Clean { .. } => Ok(()), // do nothing
            CtxOut::Verify { path, rem, .. } => {
                if *rem != 0 {
                    return Err(make_verify_report!(self, path));
                }
                Ok(())
            }
        }
    }

    pub fn make_error(&self, kind: PpErrorKind) -> PpError {
        make_error!(self, kind)
    }

    fn make_error_with_kind(file: String, kind: PpErrorKind) -> PpError {
        PpError {
            kind,
            line: 0,
            file,
        }
    }
}

/// Macro needed because of borrowing constraints.
macro_rules! make_error {
    ($self:ident, $kind:expr) => {
        PpError {
            kind: $kind,
            line: $self.cur_line,
            file: $self.input_path.clone(),
        }
    };
}
use make_error;

macro_rules! make_verify_report {
    ($self:ident, $path:expr) => {
        Report::new(make_error!($self, PpErrorKind::VerifyOutput)).attach_printable(format!(
            "`{}` is different from fresh output.",
            normalize_path(&$path.display().to_string())
        ))
    };
}
use make_verify_report;

/// Output context, which depends on the mode.
#[derive(Debug)]
enum CtxOut {
    /// Build mode.
    ///
    /// Write to the output file.
    Build {
        /// Path to the output file
        path: PathBuf,
        /// Output writer
        out: BufWriter<File>,
    },
    /// Build mode in memory
    ///
    /// Write to the output file.
    InMemoryBuild {
        /// Path to the output file
        path: PathBuf,
        /// Output buffer
        out: String,
    },
    /// Clean mode.
    ///
    /// Delete the output file and temporary files and do nothing when writing
    Clean,
    /// Verify mode.
    ///
    /// Read existing file and verify that it is the same as the fresh output
    Verify {
        path: PathBuf,
        out: BufReader<File>,
        rem: u64,
    },
}

impl CtxOut {
    fn new<P>(mode: Mode, input_path: &str, output_path: &P) -> Result<Self, PpError>
    where
        P: AsRef<Path>,
    {
        match mode {
            Mode::Build => {
                let out = File::create(output_path)
                    .change_context_lazy(|| {
                        IOCtx::make_error_with_kind(input_path.to_string(), PpErrorKind::OpenFile)
                    })
                    .attach_printable_lazy(|| {
                        format!(
                            "could not create output file: `{}`",
                            normalize_path(&output_path.as_ref().display().to_string())
                        )
                    })
                    .map(BufWriter::new)?;
                Ok(Self::Build {
                    out,
                    path: output_path.as_ref().to_path_buf(),
                })
            }
            Mode::InMemoryBuild => Ok(Self::InMemoryBuild {
                out: String::new(),
                path: output_path.as_ref().to_path_buf(),
            }),
            Mode::Clean => {
                let p = output_path.as_ref();
                if p.exists() {
                    fs::remove_file(p)
                        .change_context_lazy(|| {
                            IOCtx::make_error_with_kind(
                                input_path.to_string(),
                                PpErrorKind::DeleteFile,
                            )
                        })
                        .attach_printable_lazy(|| {
                            format!(
                                "could not remove file: `{}`",
                                normalize_path(&p.display().to_string())
                            )
                        })?;
                }
                Ok(Self::Clean)
            }
            Mode::Verify => {
                let p = output_path.as_ref();
                if !p.exists() {
                    return Err(Report::new(IOCtx::make_error_with_kind(
                        input_path.to_string(),
                        PpErrorKind::VerifyOutput,
                    ))
                    .attach_printable(format!(
                        "file `{}` does not exist.",
                        normalize_path(&p.display().to_string())
                    )));
                }
                let len = fs::metadata(p)
                    .change_context_lazy(|| {
                        IOCtx::make_error_with_kind(input_path.to_string(), PpErrorKind::OpenFile)
                    })
                    .attach_printable_lazy(|| {
                        format!(
                            "could not get metadata for output file: `{}`",
                            normalize_path(&p.display().to_string())
                        )
                    })?
                    .len();
                log::debug!("found output to verify, file size: {}", len);
                let out = File::open(output_path)
                    .change_context_lazy(|| {
                        IOCtx::make_error_with_kind(input_path.to_string(), PpErrorKind::OpenFile)
                    })
                    .attach_printable_lazy(|| {
                        format!(
                            "could not open output file: `{}`",
                            normalize_path(&p.display().to_string())
                        )
                    })
                    .map(BufReader::new)?;
                Ok(Self::Verify {
                    out,
                    rem: len,
                    path: p.to_path_buf(),
                })
            }
        }
    }
}
