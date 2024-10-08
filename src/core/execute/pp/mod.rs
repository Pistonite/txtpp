use crate::core::{Mode, TagState};
use crate::error::{PpError, PpErrorKind};
use crate::fs::{AbsPath, IOCtx, Shell, TxtppPath};
use error_stack::{Report, Result, ResultExt};
use std::path::PathBuf;

mod directive;
pub use directive::*;

/// Preprocess the txtpp file
pub fn preprocess(
    shell: &Shell,
    input_file: &AbsPath,
    mode: Mode,
    is_first_pass: bool,
    trailing_newline: bool,
) -> Result<PpResult, PpError> {
    Pp::run(input_file, shell, mode, is_first_pass, trailing_newline)
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
    execute_tail_line: Option<String>,
}

impl<'a> Pp<'a> {
    fn run(
        input_file: &AbsPath,
        shell: &'a Shell,
        mode: Mode,
        is_first_pass: bool,
        trailing_newline: bool,
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
            execute_tail_line: None,
        }
        .run_internal(trailing_newline)
    }

    fn run_internal(mut self, trailing_newline: bool) -> Result<PpResult, PpError> {
        let mut add_newline_before_next_output = false;
        // read txtpp file line by line
        loop {
            let line = self.get_next_line()?;

            let (to_write, has_tail) = match self
                .iterate_directive(line)
                .ignore_err_if_cleaning(&self.mode, || IterDirectiveResult::None("".to_string()))?
            {
                IterDirectiveResult::Break => break,
                IterDirectiveResult::LineTaken => {
                    // Don't write the line
                    (None, false)
                }
                IterDirectiveResult::None(line) => {
                    // Writing the line from source to output
                    let line = if self.pp_mode.is_execute() {
                        self.tag_state.inject_tags(&line, self.context.line_ending)
                    } else {
                        line
                    };
                    (Some(line), false)
                }
                IterDirectiveResult::Execute(d, line) => {
                    let whitespaces = d.whitespaces.clone();
                    let d_str = format!("for `{d}`");
                    let directive_output = if let Some(raw_output) = self
                        .execute_directive(d)
                        .map_err(|e| e.attach_printable(d_str))?
                    {
                        log::debug!("directive output: {raw_output:?}");
                        if self.tag_state.try_store(&raw_output).is_err() {
                            Some(self.format_directive_output(
                                &whitespaces,
                                raw_output.lines(),
                                raw_output.ends_with('\n'),
                            ))
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    let has_tail = if line.is_some() {
                        self.execute_tail_line = line;
                        true
                    } else {
                        false
                    };

                    (directive_output, has_tail)
                }
            };

            if self.pp_mode.is_execute() {
                if let Some(x) = to_write {
                    if add_newline_before_next_output {
                        self.context.write_output(self.context.line_ending)?;
                    }
                    add_newline_before_next_output = !has_tail;
                    self.context.write_output(&x)?;
                }
            }
        }

        if let PpMode::CollectDeps(deps) = self.pp_mode {
            return Ok(PpResult::HasDeps(self.input_file, deps));
        }

        if self.tag_state.has_tags() && !matches!(self.mode, Mode::Clean) {
            return Err(
                Report::from(self.context.make_error(PpErrorKind::Directive))
                    .attach_printable("Unused tag(s) found at the end of the file. Please make sure all created tags are used up.")
                    .attach_printable(format!("tags: {}", self.tag_state))
            );
        }

        if add_newline_before_next_output && trailing_newline {
            self.context.write_output(self.context.line_ending)?;
        }

        self.context.done()?;

        Ok(PpResult::Ok(self.input_file))
    }

    /// retrieve the next line
    fn get_next_line(&mut self) -> Result<Option<String>, PpError> {
        if self.execute_tail_line.is_some() {
            return Ok(self.execute_tail_line.take());
        }
        let line = match self.context.next_line() {
            Some(line) => Some(line?),
            None => None,
        };
        Ok(line)
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
                                        .attach_printable("multi-line directive must have a prefix. Trying adding a non-empty string before `TXTPP#`")
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
            // Ignore error if in clean mode
            let _ = self.execute_in_clean_mode(d);
            return Ok(None);
        }
        let d = match self.execute_in_collect_deps_mode(d)? {
            Some(d) => d,
            None => return Ok(None),
        };

        let raw_output = match d.directive_type {
            DirectiveType::Empty | DirectiveType::After => {
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
                let include_file = self
                    .context
                    .work_dir
                    .try_resolve(&arg, false)
                    .map_err(|e| {
                        e.change_context(self.context.make_error(PpErrorKind::Directive))
                            .attach_printable(format!("could not open include file: `{arg}`"))
                    })?;
                let output = std::fs::read_to_string(&include_file)
                    .change_context_lazy(|| self.context.make_error(PpErrorKind::Directive))
                    .attach_printable_lazy(|| {
                        format!("could not read include file: `{include_file}`")
                    })?;
                log::debug!("include file content: {output:?}");
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
            DirectiveType::Write => Some(d.args.join("\n")),
        };
        Ok(raw_output)
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
        if matches!(
            d.directive_type,
            DirectiveType::Include | DirectiveType::After
        ) {
            let arg = d.args.first().cloned().unwrap_or_default();
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
        // if we are already collecting deps, don't execute the directive
        if let PpMode::CollectDeps(_) = self.pp_mode {
            return Ok(None);
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

        if PathBuf::from(export_file).is_txtpp_file() {
            return Err(Report::new(self.context.make_error(PpErrorKind::Directive))
                .attach_printable(format!(
                "invalid temp directive: export file path cannot be a txtpp file: `{export_file}`"
            )));
        }

        if is_clean {
            return self.context.write_temp_file(export_file, "");
        }
        // We force trailing newline if the file is not empty
        let contents = self.format_directive_output("", args.iter().skip(1), false);
        self.context.write_temp_file(export_file, &contents)
    }

    fn format_directive_output(
        &mut self,
        whitespaces: &str,
        raw_output: impl Iterator<Item = impl AsRef<str>>,
        has_trailing_newline: bool,
    ) -> String {
        let mut output = raw_output
            .map(|s| format!("{whitespaces}{line}", line = s.as_ref()))
            .collect::<Vec<_>>()
            .join(self.context.line_ending);
        if has_trailing_newline {
            output.push_str(self.context.line_ending);
        }
        output
    }
}

trait IgnoreIfCleaning {
    type Output;
    fn ignore_err_if_cleaning<F>(self, mode: &Mode, f: F) -> Result<Self::Output, PpError>
    where
        Self: Sized,
        F: FnOnce() -> Self::Output;
}

impl<T> IgnoreIfCleaning for Result<T, PpError> {
    type Output = T;
    fn ignore_err_if_cleaning<F>(self, mode: &Mode, f: F) -> Result<T, PpError>
    where
        F: FnOnce() -> Self::Output,
    {
        if self.is_err() && matches!(mode, Mode::Clean) {
            Ok(f())
        } else {
            self
        }
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

/// Processing result
#[derive(Debug)]
pub enum PpResult {
    /// File was processed successfully
    Ok(AbsPath),
    /// Dependency is found
    HasDeps(AbsPath, Vec<AbsPath>),
}
