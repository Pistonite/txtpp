use std::fmt::Write;
use std::path::PathBuf;

use crate::core::{Mode, ReplaceLineEnding, TagState};
use crate::error::{PpError, PpErrorKind};
use crate::fs::{AbsPath, IOCtx, Shell, TxtppPath};
use error_stack::{IntoReport, Report, Result};

mod directive;
pub use directive::*;

/// Preprocess the txtpp file
///
/// # Arguments
/// shell: The shell to use
/// base: The directory of the current txtpp process as absolute path
/// mode: whether is in verification mode
/// txtpp_file: the txtpp file to preprocess, as relative path to base
///
pub fn do_preprocess(
    shell: &Shell,
    input_file: &AbsPath,
    mode: Mode,
    is_first_pass: bool,
) -> Result<PpResult, PpError> {
    Pp::run(input_file, shell, mode, is_first_pass)
}

/// Preprocesser runtime
struct Pp<'a> {
    shell: &'a Shell,
    input_file: AbsPath,
    mode: Mode,
    context: IOCtx,
    cur_directive: Option<Directive>,
    tag_state: TagState,
    pp_mode: PpMode,
}

impl<'a> Pp<'a> {
    fn run(
        input_file: &AbsPath,
        shell: &'a Shell,
        mode: Mode,
        is_first_pass: bool,
    ) -> Result<PpResult, PpError> {
        let context = IOCtx::new(input_file, mode.clone())?;
        Self {
            shell,
            input_file: input_file.clone(),
            mode,
            context,
            cur_directive: None,
            tag_state: TagState::new(),
            pp_mode: if is_first_pass {
                PpMode::FirstPassExecute
            } else {
                PpMode::Execute
            },
        }
        .run_internal()
    }

    /// Preprocess the txtpp file
    ///
    /// # Arguments
    /// shell: The shell to use
    /// base: The directory of the current txtpp process as absolute path
    /// mode: whether is in verification mode
    /// txtpp_file: the txtpp file to preprocess, as relative path to base
    ///
    fn run_internal(mut self) -> Result<PpResult, PpError> {
        // read txtpp file line by line
        loop {
            let line = match self.context.next_line() {
                Some(line) => Some(line?),
                None => None,
            };

            let to_write = match self.iterate_directive(line)? {
                IterDirectiveResult::Break => break,
                IterDirectiveResult::LineTaken => {
                    // Don't write the line
                    None
                }
                IterDirectiveResult::None(line) => {
                    // Writing the line from source to output
                    let line = if self.pp_mode.is_execute() {
                        self.tag_state.inject_tags(&line, self.context.line_ending)
                    } else {
                        line
                    };
                    Some(line)
                }
                IterDirectiveResult::Execute(d, line) => {
                    let directive_output = self.execute_directive(d)?.and_then(|s| {
                        let s = s.replace_line_ending(self.context.line_ending, true);
                        if self.tag_state.try_store(&s).is_err() {
                            Some(s)
                        } else {
                            None
                        }
                    });
                    let line = if let Some(line) = line {
                        match Directive::detect_from(&line) {
                            Some(d) => {
                                self.cur_directive = Some(d);
                                None
                            }
                            None => Some(line),
                        }
                    } else {
                        line
                    };
                    let line = if self.pp_mode.is_execute() {
                        line.map(|line| self.tag_state.inject_tags(&line, self.context.line_ending))
                    } else {
                        line
                    };
                    let output = match (line, directive_output) {
                        (Some(line), Some(directive_output)) => {
                            Some(format!("{}{}", directive_output, line))
                        }
                        (Some(line), None) => Some(line),
                        (None, Some(directive_output)) => Some(directive_output),
                        (None, None) => None,
                    };

                    output
                }
            };

            if self.pp_mode.is_execute() {
                if let Some(x) = to_write {
                    self.context.write_output(&x)?;
                }
            }
            //self.cur_directive = next_directive;
        }

        if let PpMode::CollectDeps(deps) = self.pp_mode {
            return Ok(PpResult::HasDeps(self.input_file, deps));
        }

        self.context.done()?;

        Ok(PpResult::Ok(self.input_file))
    }

    /// Update the directive and line based on the current directive and the next line
    fn iterate_directive(&mut self, line: Option<String>) -> Result<IterDirectiveResult, PpError> {
        let next = match line {
            None => {
                // End of file, execute the current directive
                match self.cur_directive.take() {
                    Some(d) => IterDirectiveResult::Execute(d, None),
                    None => IterDirectiveResult::Break,
                }
            }
            Some(line) => match self.cur_directive.take() {
                // Detect new directive
                None => match Directive::detect_from(&line) {
                        Some(d) => {
                            // make sure multi-line directives don't have empty prefix
                            if d.directive_type.supports_multi_line() && d.prefix.is_empty() {
                                return Err(
                                    Report::from(self.context.make_error(PpErrorKind::Directive))
                                        .attach_printable("multi-line directive must have a prefix."),
                                );
                            }
                            // Detected, remove this line
                            self.cur_directive = Some(d);
                            IterDirectiveResult::LineTaken
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
                        self.cur_directive = Some(d);
                        IterDirectiveResult::LineTaken
                    },
                    Err(_) => {
                        // Not added, keep this line, and ready to execute the directive
                        IterDirectiveResult::Execute(d, Some(line))
                    }
                },
            },
        };

        log::debug!("next directive: {:?}", next);
        Ok(next)
    }

    /// Execute the directive and return the output from the directive
    fn execute_directive(&mut self, d: Directive) -> Result<Option<String>, PpError> {
        if let Mode::Clean = self.mode {
            self.execute_in_clean_mode(d)?;
            return Ok(None);
        }
        let d = match self.execute_in_collect_deps_mode(d)? {
            Some(d) => d,
            None => return Ok(None),
        };

        let raw_output = match d.directive_type {
            DirectiveType::Empty => {
                // do nothing (consume the line)
                None
            }
            DirectiveType::Run => {
                let command = d.args.join(" ");
                let output = self
                    .shell
                    .run(&command, &self.context.work_dir, &self.context.input_path)
                    .map_err(|e| {
                        e.change_context(self.context.make_error(PpErrorKind::Directive))
                            .attach_printable(format!("failed to run command: `{command}`."))
                    })?;
                Some(output)
            }
            DirectiveType::Include => {
                let arg = d.args.into_iter().next().unwrap_or_default();
                //let include_path = PathBuf::from(&arg);
                //let include_path = self.context.work_dir.as_path().join(include_path);
                // See if we need to store the dependency and come back later
                // if is_first_pass {
                //     if let Some(x) = include_path.get_txtpp_file() {
                //         log::debug!("found dependency: {}", x.display());
                //         *executing = false;
                //         let p_abs = self.context.work_dir.share_base(x).map_err(|e| {
                //             e.change_context(self.context.make_error(PpErrorKind::Directive))
                //                 .attach_printable(format!(
                //                     "could not resolve include file: `{}`",
                //                     include_path.display()
                //                 ))
                //         })?;
                //         deps.push(p_abs);
                //     }
                // }
                let include_file = self
                    .context
                    .work_dir
                    .try_resolve(&arg, false)
                    .map_err(|e| {
                        e.change_context(self.context.make_error(PpErrorKind::Directive))
                            .attach_printable(format!("could not open include file: `{arg}`"))
                    })?;
                let output = std::fs::read_to_string(&include_file)
                    .into_report()
                    .map_err(|e| {
                        e.change_context(self.context.make_error(PpErrorKind::Directive))
                            .attach_printable(format!(
                                "could not read include file: `{include_file}`"
                            ))
                    })?;
                Some(output)
            }
            DirectiveType::Temp => {
                self.execute_directive_temp(d.args, false)?;

                None
            }
            DirectiveType::Tag => {
                let tag_name = d.args.into_iter().next().unwrap_or_default();
                self.tag_state.create(&tag_name).map_err(|e| {
                    e.change_context(self.context.make_error(PpErrorKind::Directive))
                        .attach_printable(format!("could not create tag: `{tag_name}`"))
                })?;
                None
            }
            DirectiveType::Write => Some(d.args.into_iter().skip(1).collect::<Vec<_>>().join("\n")),
        };
        if let Some(output) = raw_output {
            Ok(Some(
                self.format_directive_output(&d.whitespaces, output.lines())?,
            ))
        } else {
            Ok(None)
        }
    }

    /// Execute the directive in clean mode
    fn execute_in_clean_mode(&mut self, d: Directive) -> Result<(), PpError> {
        if let DirectiveType::Temp = d.directive_type {
            self.execute_directive_temp(d.args, true)?;
        }
        Ok(())
    }

    /// Execute the directive in collect dep mode
    fn execute_in_collect_deps_mode(&mut self, d: Directive) -> Result<Option<Directive>, PpError> {
        if let PpMode::Execute = self.pp_mode {
            // never collect deps in execute mode
            return Ok(Some(d));
        }
        if let DirectiveType::Include = d.directive_type {
            let arg = d.args.iter().next().cloned().unwrap_or_default();
            let include_path = PathBuf::from(&arg);
            // We use join instead of share_base because the dependency might not exist
            let include_path = self.context.work_dir.as_path().join(include_path);
            // See if we need to store the dependency and come back later
            if let Some(x) = include_path.get_txtpp_file() {
                log::debug!("found dependency: {}", x.display());
                let p_abs = self.context.work_dir.share_base(x).map_err(|e| {
                    e.change_context(self.context.make_error(PpErrorKind::Directive))
                        .attach_printable(format!(
                            "could not resolve include file: `{}`",
                            include_path.display()
                        ))
                })?;
                match &mut self.pp_mode {
                    PpMode::CollectDeps(deps) => {
                        deps.push(p_abs);
                    }
                    PpMode::FirstPassExecute => {
                        self.pp_mode = PpMode::CollectDeps(vec![p_abs]);
                    }
                    _ => unreachable!(),
                }
                return Ok(None);
            }
        }
        Ok(Some(d))
    }

    fn execute_directive_temp(&mut self, args: Vec<String>, is_clean: bool) -> Result<(), PpError> {
        let export_file = match args.first() {
            Some(p) => p,
            None => {
                return Err(Report::new(self.context.make_error(PpErrorKind::Directive))
                    .attach_printable("invalid temp directive: no export file path specified"));
            }
        };
        if is_clean {
            return self.context.write_temp_file(export_file, "");
        }
        let contents = self.format_directive_output("", args.iter().skip(1))?;
        self.context.write_temp_file(export_file, &contents)
    }

    fn format_directive_output(
        &mut self,
        whitespaces: &str,
        raw_output: impl Iterator<Item = impl AsRef<str>>,
    ) -> Result<String, PpError> {
        let mut output = String::new();
        for line in raw_output {
            write!(
                output,
                "{whitespaces}{line}{line_ending}",
                line = line.as_ref(),
                line_ending = self.context.line_ending
            )
            .into_report()
            .map_err(|e| {
                e.change_context(self.context.make_error(PpErrorKind::Directive))
                    .attach_printable("could not format output")
            })?;
        }
        Ok(output)
    }
}

/// Result of reading the next line of a directive
#[derive(Debug)]
enum IterDirectiveResult {
    /// Stop processing
    Break,
    /// The directive is none and the line is not a directive
    None(String),
    /// The next line is taken by the directive
    LineTaken,
    /// The directive is complete and should be executed
    Execute(Directive, Option<String>),
}
enum PpMode {
    /// Execute until the first dep, and turn into `CollectDeps`
    FirstPassExecute,
    /// Execute, don't collect deps
    Execute,
    /// Just collect deps
    CollectDeps(Vec<AbsPath>),
}

impl PpMode {
    fn is_execute(&self) -> bool {
        matches!(self, PpMode::Execute | PpMode::FirstPassExecute)
    }
}

#[derive(Debug)]
pub enum PpResult {
    Ok(AbsPath),
    HasDeps(AbsPath, Vec<AbsPath>),
}
