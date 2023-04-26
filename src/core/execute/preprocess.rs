use crate::core::{Directive, ReplaceLineEnding, TagState, Mode};
use crate::fs::{AbsPath, Context, Shell};
use error_stack::{Result, Report};
use std::error;
use std::fmt;

/// Preprocess the txtpp file
///
/// # Arguments
/// shell: The shell to use
/// base: The directory of the current txtpp process as absolute path
/// is_verifying: whether is in verification mode
/// txtpp_file: the txtpp file to preprocess, as relative path to base
///
pub fn do_preprocess(
    shell: &Shell,
    input_file: &AbsPath,
    mode: Mode,
    is_first_pass: bool,
) -> Result<PreprocessResult, PreprocessError> {
    // Open the preprocessor context
    let mut context = Context::new(input_file, mode == Mode::Build).map_err(|e| {
        e.change_context(PreprocessError::File(input_file.clone()))
            .attach_printable("failed to initialize context.")
    })?;

    if mode == Mode::Clean {
        context.clean().map_err(|e| {
            e.change_context(PreprocessError::File(input_file.clone()))
                .attach_printable("failed to clean.")
        })?;
        return Ok(PreprocessResult::Ok(input_file.clone()));
    }

    let mut current_directive = None;
    let mut executing = true;
    let mut deps = vec![];
    let mut tag_state = TagState::new();
    // read txtpp file line by line
    loop {
        let line = match context.next_line() {
            Some(Ok(line)) => Some(line),
            Some(Err(e)) => {
                return Err(e
                    .attach_printable("fail to read input file.")
                    .change_context(PreprocessError::File(input_file.clone())))
            }
            None => None
        };

        log::debug!("line: {:?}", line);

        let (next_directive, to_write) = match iterate_directive(current_directive, line) {
            IterDirectiveResult::Break => break,
            IterDirectiveResult::None(line) => {
                // Writing the line from source to output
                let line = if executing {
                    tag_state.inject_tags(&line, context.line_ending)
                } else {
                    line
                };
                (None, Some(line))
            }
            IterDirectiveResult::LineTaken(d) => {
                // Don't write the line
                (Some(d), None)
            }
            IterDirectiveResult::Execute(d, line) => {
                let directive_output = d
                    .execute(
                        shell,
                        &mut context,
                        &mut executing,
                        &mut deps,
                        &mut tag_state,
                        is_first_pass,
                    )
                    .map_err(|e| {
                        e.change_context(PreprocessError::Directive(input_file.clone()))
                            .attach_printable("failed to execute directive.")
                    })?
                    .and_then(|s| {
                        let mut s = s.replace_line_ending(context.line_ending, true);
                        if tag_state.try_store(&s).is_err() {
                            if let Some(l) = line {
                                let l = if executing {
                                    tag_state.inject_tags(&l, context.line_ending)
                                } else {
                                    l
                                };
                                s.push_str(&l);
                            }
                            
                            Some(s)
                        } else {
                            None
                        }
                    });

                (None, directive_output)
            }
        };

        // make sure multi-line directives don't have empty prefix
        if let Some(d) = &next_directive {
            if d.directive_type.supports_multi_line() && d.prefix.is_empty() {
                return Err(Report::from(PreprocessError::Directive(input_file.clone()))
                    .attach_printable("multi-line directive must have a prefix."));
            }
        }

        if executing {
            if let Some(x) = to_write {
                context.write(&x).map_err(|e| {
                    e.change_context(PreprocessError::File(input_file.clone()))
                        .attach_printable("could not write to output file")
                })?;
            }
        }
        current_directive = next_directive;
    }

    if !executing {
        return Ok(PreprocessResult::HasDeps(input_file.clone(), deps));
    }

    context.done().map_err(|e| {
        e.change_context(PreprocessError::File(input_file.clone()))
            .attach_printable("could not save output file")
    })?;

    if mode == Mode::Verify {
        // check if the thing exist and is the same
        context.verify().map_err(|e| {
            e.change_context(PreprocessError::Verify(input_file.clone()))
                .attach_printable("output is not up to date.")
        })?;
    }

    Ok(PreprocessResult::Ok(input_file.clone()))
}

/// Update the directive and line based on the current directive and the next line
fn iterate_directive(current_directive: Option<Directive>, line: Option<String>) -> IterDirectiveResult {
    match line {
        None => {
            // End of file, execute the current directive
            match current_directive {
                Some(d) => IterDirectiveResult::Execute(d, None),
                None => IterDirectiveResult::Break,
            }
        },
        Some(line) => match current_directive {
            // Detect new directive
            None => match Directive::detect_from(&line) {
                    Some(d) => {
                        // Detected, remove this line
                        IterDirectiveResult::LineTaken(d)
                    },
                    None => {
                        // Not detected, keep this line
                        IterDirectiveResult::None(line)
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
                    IterDirectiveResult::Execute(d, Some(line))
                }
            },
        }
    }
    
}

#[derive(Debug)]
pub enum PreprocessError {
    File(AbsPath),
    Directive(AbsPath),
    Verify(AbsPath),
}

impl fmt::Display for PreprocessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PreprocessError::File(p) => write!(f, "File error processing {p}"),
            PreprocessError::Directive(p) => write!(f, "Directive error processing {p}"),
            PreprocessError::Verify(p) => write!(f, "{p} is not up to date"),
        }
    }
}

impl error::Error for PreprocessError {}

/// Result of reading the next line of a directive
enum IterDirectiveResult {
    /// Stop processing
    Break,
    /// The directive is none and the line is not a directive
    None(String),
    /// The next line is taken by the directive
    LineTaken(Directive),
    /// The directive is complete and should be executed
    Execute(Directive, Option<String>),
}

pub enum PreprocessResult {
    Ok(AbsPath),
    HasDeps(AbsPath, Vec<AbsPath>),
}
