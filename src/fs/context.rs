use crate::fs::{AbsPath, TxtppPath, GetLineEnding};
use error_stack::{IntoReport, Report, Result};
use std::error;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Lines, Write};
use std::path::PathBuf;


/// Context for preprocessing a txtpp file.
///
/// This is an IO wrapper for reading from txtpp file and writing to the output file.
pub struct Context {
    input: Lines<BufReader<File>>,
    out: CtxOut,
    output_path: PathBuf,
    pub work_dir: AbsPath,
    pub line_ending: &'static str,
    pub input_path: String,
}

impl Context {
    /// Create a new context for the input txtpp file.
    pub fn new(input_file: &AbsPath, create_file: bool) -> Result<Self, CtxError> {
        let input_path = input_file.to_string();
        
        let line_ending = input_file.get_line_ending();
        let r = File::open(input_file)
            .map(BufReader::new)
            .into_report()
            .map_err(|e| {
                e.change_context(CtxError::OpenInputFile)
                    .attach_printable(format!("could not open input file: {input_path}."))
            })?;

        let output_path = input_file.as_path_buf().trim_txtpp().map_err(|e|{
            e.change_context(CtxError::OpenInputFile)
                .attach_printable(format!("could not resolve output path for input file: {input_path}."))
        })?;

        let out = if !create_file {
            CtxOut::Str(String::new())
        } else {
            let file_writer = File::create(&output_path)
                .into_report()
                .map_err(|e| {
                    e.change_context(CtxError::OpenOutputFile)
                        .attach_printable(format!("could not open output file: {}", output_path.display()))
                })
                .map(BufWriter::new)?;
            CtxOut::File(file_writer)
        };

        let work_dir = input_file.parent().map_err(|e| {
            e.change_context(CtxError::OpenInputFile)
                .attach_printable(format!("cannot get parent for input file: {}", input_file))
        })?;

        Ok(Self {
            input: r.lines(),
            out,
            output_path,
            work_dir,
            line_ending,
            input_path,
        })
    }

    /// Get the next line from the input file.
    pub fn next_line(&mut self) -> Option<Result<String, CtxError>> {
        self.input.next().map(|line| {
            line.into_report().map_err(|e| {
                e.change_context(CtxError::ReadInputFile)
                    .attach_printable("cannot read next line")
            })
        })
    }

    pub fn write(&mut self, output: &str) -> Result<(), CtxError> {
        self.out.write(output).into_report().map_err(|e| {
            e.change_context(CtxError::WriteOutputFile)
                .attach_printable("cannot write to output file.")
        })
    }
    pub fn done(&mut self) -> Result<(), CtxError> {
        self.out.done().into_report().map_err(|e| {
            e.change_context(CtxError::WriteOutputFile)
                .attach_printable("cannot save output file.")
        })
    }

    pub fn verify(self) -> Result<(), CtxError> {
        let p = &self.output_path;
        if !p.exists() {
            return Err(Report::new(CtxError::Mismatch)
                .attach_printable(format!("file `{}` does not exist.", p.display())));
        }
        let expected = match self.out {
            CtxOut::Str(s) => s,
            CtxOut::File(_) => {
                return Err(Report::new(CtxError::Other).attach_printable(
                    "verifying output content but output is a stream",
                ));
            }
        };
        let actual = std::fs::read_to_string(p).into_report().map_err(|e| {
            e.change_context(CtxError::OpenOutputFile)
                .attach_printable(format!("could not open file: `{}`", p.display()))
        })?;
        if expected != actual {
            Err(Report::new(CtxError::Mismatch).attach_printable(format!(
                "file `{}` is different from fresh output.",
                p.display()
            )))
        } else {
            Ok(())
        }
    }

    pub fn clean(self) -> Result<(), CtxError> {
        let p = &self.output_path;
        if p.exists() {
            std::fs::remove_file(p).into_report().map_err(|e|{
                e.change_context(CtxError::WriteOutputFile)
                    .attach_printable(format!("could not remove file: `{}`", p.display()))
            })?;
        }

        Ok(())
    }
}

enum CtxOut {
    Str(String),
    File(BufWriter<File>),
}

impl CtxOut {
    pub fn write(&mut self, output: &str) -> io::Result<()> {
        match self {
            Self::Str(s) => {
                s.push_str(output);
                Ok(())
            }
            Self::File(w) => w.write_all(output.as_bytes()),
        }
    }
    pub fn done(&mut self) -> io::Result<()> {
        if let Self::File(f) = self {
            f.flush()
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
pub enum CtxError {
    OpenInputFile,
    ReadInputFile,
    OpenOutputFile,
    WriteOutputFile,
    Mismatch,
    Other,
}

impl fmt::Display for CtxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::OpenInputFile => write!(f, "Could not open input file."),
            Self::ReadInputFile => write!(f, "Could not read input file."),
            Self::OpenOutputFile => write!(f, "Could not open output file."),
            Self::WriteOutputFile => write!(f, "Could not write to output file."),
            Self::Mismatch => write!(f, "Output file content does not match fresh run."),
            Self::Other => write!(f, "Internal error."),
        }
    }
}

impl error::Error for CtxError {}
