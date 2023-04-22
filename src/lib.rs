mod core;
pub use crate::core::{execute, Config};
use crate::core::*;
mod path;
use path::*;
mod fs;
use std::collections::HashMap;
use std::env::current_dir;
use std::error;
use std::fs::{File};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

mod constants;
mod shell;
use shell::Shell;

mod write;
use error_stack::{IntoReport, Report, Result};
use write::WriteWrapper;

use crate::fs::GetLineEnding;



// pub fn build() -> Result<(), PreprocessError> {
    
// }

// pub fn verify() -> Result<(), PreprocessError> {
    
// }

// pub fn clean() -> Result<(), PreprocessError> {
    
// }


#[derive(Debug)]
enum PreprocessError {
    InternalInvalid,
    InvalidInput,
    OpenInputFile,
    ReadInputFile,
    OpenOutputFile,
    WriteOutputFile,
    Directive,
    Mismatch
}

impl std::fmt::Display for PreprocessError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Could not process input file.")
    }
}

impl error::Error for PreprocessError {}

enum IterDirectiveResult {
    LineTaken(Directive),
    NotDirective(String),
    Execute(Directive, String),
}

enum PreprocessResult {
    Ok,
    HasDeps(Vec<AbsPath>),
}


/// Preprocess the txtpp file
///
/// # Arguments
/// shell: The shell to use
/// base: The directory of the current txtpp process as absolute path
/// is_checking: whether is in verification mode
/// txtpp_file: the txtpp file to preprocess, as relative path to base
///
fn do_preprocess(
    shell: &Shell,
    is_checking: bool,
    input_file: &AbsPath,
    is_first_pass: bool,
) -> Result<PreprocessResult, PreprocessError> {
    // Open the preprocessor file
    let input_file_path = input_file.as_ref().display().to_string();
    let output_file = input_file.trim_txtpp().into_report().map_err(|e| {
        e.attach_printable(format!("Invalid input file: {}", input_file))
            .change_context(PreprocessError::InternalInvalid)
    })?;

    let line_ending = input_file.get_line_ending();
    let r = File::open(&input_file)
        .map(BufReader::new)
        .into_report()
        .map_err(|e| {
            e.attach_printable(format!("Could not open input file: {}", input_file))
                .change_context(PreprocessError::OpenInputFile)
        })?;


    let mut w = WriteWrapper::new(is_checking, &output_file).into_report().map_err(|e| {
        e.attach_printable(format!("Could not open output file: {}", output_file))
            .change_context(PreprocessError::OpenOutputFile)
    })?;

    let working_directory = input_file.parent().into_report().map_err(|e| {
        e.attach_printable(format!("Cannot get working directory for input file: {}", input_file))
            .change_context(PreprocessError::InvalidInput)
    })?;
    
    let mut current_directive: Option<Directive> = None;
    let mut executing = true;
    let mut directive_index = 0;
    let mut deps = Vec::new();
    let mut tag_state = TagState::new();
    // read txtpp file line by line
    for line in r.lines() {
        let line = line.into_report().map_err(|e| {
            e.attach_printable(format!("Could not read input file: {}", input_file))
                .change_context(PreprocessError::ReadInputFile)
        })?;
        let result = match current_directive {
            // Detect new directive
            None =>   match Directive::detect_from(&line) {
                    Some(d) => {
                        // Detected, remove this line
                        IterDirectiveResult::LineTaken(d)
                    },
                    None => {
                        // Not detected, keep this line
                        IterDirectiveResult::NotDirective(line)
                    }
                }
            ,
            // Append to current directive
            Some(mut d) => match d.add_line(&line) {
                Ok(_) => {
                    // Added, remove this line
                    IterDirectiveResult::LineTaken(d)
                },
                Err(_) => {
                    // Not added, keep this line, and ready to execute the directive
                    IterDirectiveResult::Execute(d, line)                    
                }
            },
        };

        let (next_directive, to_write) = match result {
            IterDirectiveResult::LineTaken(d) => {
                // Line taken, keep going
                (Some(d), None)
            },
            IterDirectiveResult::NotDirective(line) => {
                // Not directive, keep this line
                let line = if executing {
                    tag_state.inject_tags(&line, line_ending)
                } else {
                    line
                };
                (None, Some(line))
            },
            IterDirectiveResult::Execute(d, line) => {
                let directive_output  = match d .directive_type {
                    DirectiveType::Empty => {
                        // do nothing (consume the line)
                        None
                    },
                    DirectiveType::Run => {
                        if executing {
                            let command = d.args.join(" ");
                            let shell_output = shell.run(&command, &working_directory, &input_file_path).map_err(|e| {
                                e.attach_printable(format!("Failed to run command: {}", command))
                                    .change_context(PreprocessError::Directive)
                            })?;
                            Some(shell_output)
                        } else {
                            None
                        }
                    },
                    DirectiveType::Include => {
                        let include_path = PathSpec::from(d.args);
                        let include_file = working_directory.resolve(&include_path);
                        // See if we need to store the dependency and come back later
                        if is_first_pass {
                            if let Some(x) = include_file.get_txtpp_file() {
                                executing = false;
                                deps.push(x);
                            }
                        }
                        if executing {    
                            let output = std::fs::read_to_string(&include_file).into_report().map_err(|e| {
                                e.attach_printable(format!("Could not open include file: {}", include_file))
                                    .change_context(PreprocessError::Directive)
                            })?;
                            Some(output)
                        } else {
                            None
                        }                        
                    },
                    DirectiveType::Temp => {
                        if executing {
                            let export_path = match d.args.first() {
                                Some(x) => PathSpec::from(Path::new(x).to_path_buf()),
                                None => {
                                    return Err(Report::new(PreprocessError::Directive)
                                        .attach_printable(format!("Invalid temp directive: {}. Temp directives must have at least 1 argument for the export path.", line))
                                        .change_context(PreprocessError::Directive)
                                    );
                                }
                            };
                            let mut contents = String::new();
                            d.args.iter().skip(1).for_each(|s|{
                                contents.push_str(s);
                                contents.push_str(line_ending);
                            });
                            let export_file = working_directory.resolve(&export_path);
                            std::fs::write(&export_file, contents).into_report().map_err(|e| {
                                e.attach_printable(format!("Could not write file: {}", export_file))
                                    .change_context(PreprocessError::Directive)
                            })?;
                        }
                        None
                    },
                    DirectiveType::Tag => {
                        if executing {
                            let tag_name = d.args.join("");
                            tag_state.create(&tag_name).map_err(|e|{
                                e.attach_printable(format!("Could not create tag: {}", tag_name))
                                    .change_context(PreprocessError::Directive)
                            })?;
                        }
                        None
                    },
                    

                };

                directive_index += 1;

                let directive_output = directive_output.map(|s|s.replace_line_ending(line_ending, true));

                if let Some(x) = directive_output {
                    if tag_state.try_store(&x).is_err() {
                        (None, Some(x))
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                }
            }
        };
        
        if executing {
            if let Some(x) = to_write {
                w.write(&x).into_report().map_err(|e| {
                    e.attach_printable(format!("Could not write to output file: {}", output_file))
                        .change_context(PreprocessError::WriteOutputFile)
                })?;
            }
        }
        current_directive = next_directive;
    }
    if !executing {
        return Ok(PreprocessResult::HasDeps(deps));
    }

    w.done().into_report().map_err(|e| {
        e.attach_printable(format!("Could not save output file: {}", output_file))
            .change_context(PreprocessError::WriteOutputFile)
    })?;

    if is_checking {
        // check if the thing exist and is the same
        verify(&output_file, w)?;
    }

    Ok(PreprocessResult::Ok)
}

fn verify<P>(output_path: &P, w: WriteWrapper) -> Result<(), PreprocessError> where P: AsRef<Path> {
    let p: &Path = output_path.as_ref();
    if !p.exists() {
        return Err(Report::new(PreprocessError::Mismatch).attach_printable(format!(
            "File {} is not up to date: does not exist.",
            p.display()
        )));
    }
    let expected = w.get();
    let actual = std::fs::read_to_string(p).into_report().map_err(|e| {
        e.attach_printable(format!("Could not open file: {}", p.display()))
            .change_context(PreprocessError::OpenOutputFile)
    })?;
    if expected != actual {
        return Err(Report::new(PreprocessError::Mismatch).attach_printable(format!(
            "File {} is not up to date: content differs from fresh output.",
            p.display()
        )));
    } else {
        Ok(())
    }
}
