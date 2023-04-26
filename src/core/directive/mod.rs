use crate::fs::{AbsPath, Context, Shell, TxtppPath};
use error_stack::{IntoReport, Report, Result};
use std::error;
use std::fmt;
use std::path::PathBuf;
use crate::core::TagState;

mod directive_add_line;
mod directive_fmt;
mod directive_from;

/// Directive struct
///
/// This object stores information detected from line that contains a directive
#[derive(Debug, PartialEq)]
pub struct Directive {
    /// The whitespaces string before the first non-whitespace character in the line
    pub whitespaces: String,
    /// The content from the first non-whitespace to the directive prefix `TXTPP#`
    pub prefix: String,
    /// The directive type
    pub directive_type: DirectiveType,
    /// The arguments of the directive
    pub args: Vec<String>,
}

impl Directive {
    /// Create a new directive
    pub fn new(
        whitespaces: &str,
        prefix: &str,
        directive_type: DirectiveType,
        args: Vec<String>,
    ) -> Self {
        Self {
            whitespaces: whitespaces.to_string(),
            prefix: prefix.to_string(),
            directive_type,
            args,
        }
    }

    /// Execute the directive and return the output from the directive
    pub fn execute(
        self,
        shell: &Shell,
        context: &mut Context,
        executing: &mut bool,
        deps: &mut Vec<AbsPath>,
        tag_state: &mut TagState,
        is_first_pass: bool,
    ) -> Result<Option<String>, DirectiveError> {
        let raw_output = match self.directive_type {
            DirectiveType::Empty => {
                // do nothing (consume the line)
                None
            }
            DirectiveType::Run => {
                if *executing {
                    let command = self.args.join(" ");
                    let output = shell
                        .run(&command, &context.work_dir, &context.input_path)
                        .map_err(|e| {
                            e.change_context(DirectiveError)
                                .attach_printable(format!("failed to run command: {command}."))
                        })?;
                    Some(output)
                } else {
                    None
                }
            }
            DirectiveType::Include => {
                let arg = self.args.into_iter().next().unwrap_or_default();
                let include_path = PathBuf::from(&arg);
                let include_path = context.work_dir.as_path().join(include_path);
                // See if we need to store the dependency and come back later
                if is_first_pass {
                    if let Some(x) = include_path.get_txtpp_file() {
                        log::debug!("found dependency: {}", x.display());
                        *executing = false;
                        let p_abs = context.work_dir.share_base(x).map_err(|e| {
                            e.change_context(DirectiveError).attach_printable(format!(
                                "could not resolve include file: {}",
                                include_path.display()
                            ))
                        })?;
                        deps.push(p_abs);
                    }
                }
                if *executing {
                    let include_file =
                        context
                            .work_dir
                            .try_resolve(&include_path, false)
                            .map_err(|e| {
                                e.change_context(DirectiveError).attach_printable(format!(
                                    "could not open include file: {}",
                                    include_path.display()
                                ))
                            })?;
                    let output = std::fs::read_to_string(&include_file)
                        .into_report()
                        .map_err(|e| {
                            e.change_context(DirectiveError).attach_printable(format!(
                                "could not read include file: {include_file}"
                            ))
                        })?;
                   Some(output)
                } else {
                    None
                }
            }
            DirectiveType::Temp => {
                if *executing {
                    let export_path = match self.args.first() {
                        Some(p) => {
                            let p = PathBuf::from(p);
                            if p.is_dir() {
                                return Err(Report::new(DirectiveError).attach_printable(
                                    "invalid temp directive: cannot export to directory"
                                ));
                            }
                            p
                        }
                        None => {
                            return Err(Report::new(DirectiveError).attach_printable(
                                "invalid temp directive: no export file path specified"
                            ));
                        }
                    };

                    let mut contents = String::new();
                    self.args.iter().skip(1).for_each(|s| {
                        contents.push_str(s);
                        contents.push_str(context.line_ending);
                    });
                    let export_file =
                        context
                            .work_dir
                            .try_resolve(&export_path, true)
                            .map_err(|e| {
                                e.change_context(DirectiveError).attach_printable(format!(
                                    "could not resolve temp file: `{}`",
                                    export_path.display()
                                ))
                            })?;
                    std::fs::write(export_file, contents)
                        .into_report()
                        .map_err(|e| {
                            e.change_context(DirectiveError)
                                .attach_printable("could not write temp file")
                        })?;
                }
                None
            }
            DirectiveType::Tag => {
                if *executing {
                    let tag_name = self.args.into_iter().next().unwrap_or_default();
                    tag_state.create(&tag_name).map_err(|e| {
                        e.change_context(DirectiveError)
                            .attach_printable(format!("could not create tag: {tag_name}"))
                    })?;
                }
                None
            },
            DirectiveType::Write => {
                if *executing {
                    let mut contents = String::new();
                    self.args.iter().skip(1).for_each(|s| {
                        contents.push_str(s);
                        contents.push_str(context.line_ending);
                    });
                    Some(contents)
                } else {
                    None
                }
                
            }
        };
        if let Some(output) = raw_output {
            Ok(Some(Self::format_output(
                &self.whitespaces,
                &output,
                context.line_ending,
            )?))
        } else {
            Ok(None)
        }
    }
}

/// Directive type
#[derive(Debug, PartialEq)]
pub enum DirectiveType {
    /// Empty directive
    Empty,
    /// Include directive, argument is path to a file
    Include,
    /// Run directive, argument is a command
    Run,
    /// Tag directive, argument is a string
    Tag,
    /// Temp directive, argument is file content
    Temp,
    /// Write directive, argument is file content
    Write
}

impl TryFrom<&str> for DirectiveType {
    type Error = ();

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        match value {
            "" => Ok(DirectiveType::Empty),
            "include" => Ok(DirectiveType::Include),
            "run" => Ok(DirectiveType::Run),
            "tag" => Ok(DirectiveType::Tag),
            "temp" => Ok(DirectiveType::Temp),
            "write" => Ok(DirectiveType::Write),
            _ => Err(()),
        }
    }
}

impl DirectiveType {
    /// Does directive support multi-line arguments
    pub fn supports_multi_line(&self) -> bool {
        !matches!(self, DirectiveType::Include | DirectiveType::Tag)
    }
}

#[derive(Debug)]
pub struct DirectiveError;

impl fmt::Display for DirectiveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Fail to execute directive")
    }
}

impl error::Error for DirectiveError {}
