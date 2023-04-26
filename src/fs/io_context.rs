use crate::error::{PpError, PpErrorKind};
use crate::fs::{AbsPath, GetLineEnding, TxtppPath};
use crate::Mode;
use error_stack::{IntoReport, Report, Result};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Lines, Read, Write};
use std::path::{Path, PathBuf};

/// Context for processing a txtpp file.
///
/// This is an IO wrapper for reading from txtpp file and writing to the output file.
#[derive(Debug)]
pub struct IOCtx {
    input: Lines<BufReader<File>>,
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

        let line_ending = input_file.get_line_ending();
        let r = File::open(input_file)
            .map(BufReader::new)
            .into_report()
            .map_err(|e| {
                e.change_context(Self::make_error_with_kind(
                    input_path.clone(),
                    PpErrorKind::OpenFile,
                ))
                .attach_printable(format!("could not open input file: `{input_path}`"))
            })?;

        let output_path = input_file.as_path_buf().trim_txtpp().map_err(|e| {
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
            line.into_report().map_err(|e| {
                e.change_context(self.make_error(PpErrorKind::ReadFile))
                    .attach_printable("cannot read next line")
            })
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
            CtxOut::Build { path, out } => {
                out.write_all(output.as_bytes()).into_report().map_err(|e| {
                    e.change_context(make_error!(self, PpErrorKind::WriteFile))
                        .attach_printable(format!("cannot write to `{}`", path.display()))
                })
            }
            CtxOut::Clean { .. } => Ok(()), // do nothing
            CtxOut::Verify { path, out, rem } => {
                let len = output.len() as u64;
                if *rem < len {
                    return Err(make_verify_report!(self, path));
                }
                let mut buf = vec![0; output.len()];
                out.read_exact(&mut buf).into_report().map_err(|e| {
                    e.change_context(make_error!(self, PpErrorKind::ReadFile))
                        .attach_printable("cannot read from output file.")
                })?;
                if buf != output.as_bytes() {
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

        match self.out {
            CtxOut::Build { .. } | CtxOut::Verify { .. } => {
                log::debug!("writing temp file: {}", p.display());
                let export_file = self.work_dir.try_resolve(&p, true).map_err(|e| {
                    e.change_context(make_error!(self, PpErrorKind::WriteFile))
                        .attach_printable(format!("could not resolve temp file: `{}`", p.display()))
                })?;
                if export_file.as_path().is_dir() {
                    return Err(Report::new(make_error!(self, PpErrorKind::WriteFile))
                        .attach_printable(format!("cannot write to directory: `{export_file}`")));
                }
                std::fs::write(&export_file, contents)
                    .into_report()
                    .map_err(|e| {
                        e.change_context(make_error!(self, PpErrorKind::WriteFile))
                            .attach_printable(format!("could not write temp file: `{export_file}`"))
                    })
            }
            CtxOut::Clean { .. } => {
                if let Ok(export_file) = self.work_dir.try_resolve(&p, false) {
                    std::fs::remove_file(&export_file)
                        .into_report()
                        .map_err(|e| {
                            e.change_context(make_error!(self, PpErrorKind::DeleteFile))
                                .attach_printable(format!(
                                    "could not remove temp file: `{export_file}`"
                                ))
                        })?;
                }
                Ok(())
            }
        }
    }

    /// Finish
    pub fn done(mut self) -> Result<(), PpError> {
        match &mut self.out {
            CtxOut::Build { path, out } => out.flush().into_report().map_err(|e| {
                e.change_context(make_error!(self, PpErrorKind::WriteFile))
                    .attach_printable(format!("cannot write to `{}`", path.display()))
            }),
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
            $path.display()
        ))
    };
}
use make_verify_report;

/// Output context, which depends on the mode.
#[derive(Debug)]
enum CtxOut {
    Build {
        path: PathBuf,
        out: BufWriter<File>,
    },
    Clean,
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
                    .into_report()
                    .map_err(|e| {
                        e.change_context(IOCtx::make_error_with_kind(
                            input_path.to_string(),
                            PpErrorKind::OpenFile,
                        ))
                        .attach_printable(format!(
                            "could not open output file: `{}`",
                            output_path.as_ref().display()
                        ))
                    })
                    .map(BufWriter::new)?;
                Ok(Self::Build {
                    out,
                    path: output_path.as_ref().to_path_buf(),
                })
            }
            Mode::Clean => {
                let p = output_path.as_ref();
                if p.exists() {
                    std::fs::remove_file(p).into_report().map_err(|e| {
                        e.change_context(IOCtx::make_error_with_kind(
                            input_path.to_string(),
                            PpErrorKind::DeleteFile,
                        ))
                        .attach_printable(format!("could not remove file: `{}`", p.display()))
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
                    .attach_printable(format!("file `{}` does not exist.", p.display())));
                }
                let len = std::fs::metadata(p)
                    .into_report()
                    .map_err(|e| {
                        e.change_context(IOCtx::make_error_with_kind(
                            input_path.to_string(),
                            PpErrorKind::OpenFile,
                        ))
                        .attach_printable(format!(
                            "could not get metadata for output file: `{}`",
                            p.display()
                        ))
                    })?
                    .len();
                let out = File::open(output_path)
                    .into_report()
                    .map_err(|e| {
                        e.change_context(IOCtx::make_error_with_kind(
                            input_path.to_string(),
                            PpErrorKind::OpenFile,
                        ))
                        .attach_printable(format!("could not open output file: `{}`", p.display()))
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
