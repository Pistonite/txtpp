use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

mod directive;
use directive::{Directive, DirectiveType};
mod constants;
use constants::TXTPP_EXT;
mod shell;
use shell::Shell;
mod path;
use path::*;
mod write;
use write::WriteWrapper;

fn main() {
    match shell::resolve_shell("nnn") {
        Ok(s) => println!("{}", s),
        Err(e) => println!("{}", e),
    }
}

enum DepBahavior {
    /// Collect dependency (first pass)
    Collect,
    /// Ignore dependency (second pass)
    Ignore,
}

enum PrepError {
    Mismatch(String),
    /// The file has dependencies.
    HasDep(Vec<BaseRelPath>),
    /// The inputs has circular dependencies.
    CircularDep(Vec<BaseRelPath>),
    IOError(io::Error),
}

type PrepResult = Result<(), PrepError>;

impl From<io::Error> for PrepError {
    fn from(e: io::Error) -> Self {
        PrepError::IOError(e)
    }
}

/// Get the file name with the txtpp extension,
///
/// If `base/file.txtpp` exists and is a file, Returns the relative path to `base` of `file.txtpp`.
/// Otherwise, returns None.
fn get_file_txtpp(base: &BasePath, file: &BaseRelPath) -> Option<BaseRelPath> {
    let file_name: &String = file.as_ref();
    if file_name.ends_with(TXTPP_EXT) {
        return None;
    }
    let relative_path = format!("{file_name}{TXTPP_EXT}");
    let p: PathBuf = file.join_base(base).into();
    if p.is_file() {
        Some(relative_path.into())
    } else {
        None
    }
}

fn make_txtpp_context(
    base: &BasePath,
    txtpp_file: &BaseRelPath,
    is_checking: bool,
) -> io::Result<(BufReader<File>, WriteWrapper, &'static str)> {
    let txtpp_file_name: &String = txtpp_file.as_ref();
    assert!(txtpp_file_name.ends_with(TXTPP_EXT));
    let file_no_txtpp = &txtpp_file_name[..txtpp_file_name.len() - TXTPP_EXT.len()];

    let txtpp_file = base.as_ref().join(txtpp_file_name);
    let line_ending = get_line_ending_of_file(&txtpp_file)?;
    let file = base.as_ref().join(file_no_txtpp);
    let r = BufReader::new(File::open(txtpp_file)?);

    if is_checking {
        Ok((r, WriteWrapper::new_str(), line_ending))
    } else {
        let w = WriteWrapper::new_file(File::create(file)?);
        Ok((r, w, line_ending))
    }
}

fn get_line_ending_of_file(path: &Path) -> io::Result<&'static str> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buf = Vec::new();
    reader.read_until(b'\n', &mut buf)?;
    match buf.len() {
        0 => Ok(constants::OS_LINE_ENDING),
        1 => {
            if buf[0] == b'\n' {
                Ok(constants::LF)
            } else {
                Ok(constants::OS_LINE_ENDING)
            }
        }
        _ => {
            if buf[buf.len() - 1] == b'\n' {
                if buf[buf.len() - 2] == b'\r' {
                    Ok(constants::CRLF)
                } else {
                    Ok(constants::LF)
                }
            } else {
                Ok(constants::OS_LINE_ENDING)
            }
        }
    }
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
    base: &BasePath,
    is_checking: bool,
    txtpp_file: &BaseRelPath,
    dependency_behavior: DepBahavior,
) -> PrepResult {
    let (r, mut w, le) = make_txtpp_context(base, txtpp_file, is_checking)?;
    let mut directive = directive::Result::NotDirective;
    let mut outputing = true;
    let mut directive_index = 0;
    let mut deps = Vec::new();
    // read txtpp file line by line
    for line in r.lines() {
        let line = line?;
        // detect directive
        let (next_directive, output) = match directive {
            directive::Result::NotDirective => detect_directive(line, le),
            directive::Result::Directive(d) => {
                let output = match d.directive_type {
                    DirectiveType::Empty => {
                        // do nothing (consume the line)
                        None
                    }
                    DirectiveType::Include => {
                        // See if we need to store the dependency and come back later
                        let dep = if let DepBahavior::Collect = dependency_behavior {
                            if let Some(x) = get_file_txtpp(base, &d.arg.clone().into()) {
                                Some(x)
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        match dep {
                            Some(x) => {
                                outputing = false;
                                deps.push(x);
                                None
                            }
                            None => {
                                if outputing {
                                    Some(include_directive(base, txtpp_file, &d.arg)?)
                                } else {
                                    None
                                }
                            }
                        }
                    }
                    DirectiveType::Run => {
                        if outputing {
                            Some(run_directive(
                                shell,
                                base,
                                txtpp_file,
                                &d.arg,
                                directive_index,
                            )?)
                        } else {
                            None
                        }
                    }
                };
                directive_index += 1;
                let output = if let Some(x) = output {
                    Some(d.format_output(&x, le)?)
                } else {
                    None
                };

                (directive::Result::NotDirective, output)
            }
            directive::Result::NeedNextLine(d) => (d.addln(&line), None),
        };
        if outputing {
            if let Some(output) = output {
                w.write(&output)?;
            }
        }
        directive = next_directive;
    }
    if !outputing {
        return Err(PrepError::HasDep(deps));
    }

    w.done()?;

    if is_checking {
        // check if the thing exist and is the same
        verify(txtpp_file, base, w)?;
    }

    Ok(())
}

fn verify(txtpp_file: &BaseRelPath, base: &BasePath, w: WriteWrapper) -> PrepResult {
    let rel_path: &String = txtpp_file.as_ref();
    let output_path = &rel_path[..rel_path.len() - TXTPP_EXT.len()];
    let abs_path = txtpp_file.join_base(base);
    let p: &Path = abs_path.as_ref();
    if !p.exists() {
        return Err(PrepError::Mismatch(format!(
            "{} is not up to date (file not found).",
            output_path
        )));
    }
    let expected = w.get();
    let actual = fs::read_to_string(base.as_ref().join(output_path))?;
    if expected != actual {
        Err(PrepError::Mismatch(format!(
            "{} is not up to date (contents are different).",
            output_path
        )))
    } else {
        Ok(())
    }
}

fn detect_directive(line: String, le: &str) -> (directive::Result, Option<String>) {
    let d = Directive::detect_from(&line);
    if d.is_none() {
        let mut line = line;
        line.push_str(le);
        (d, Some(line))
    } else {
        (d, None)
    }
}

fn run_directive(
    shell: &Shell,
    base: &BasePath,
    txtpp_file: &BaseRelPath,
    arg: &str,
    directive_index: i32,
) -> io::Result<String> {
    let txtpp_file_path: &Path = txtpp_file.as_ref();
    let file_path = txtpp_file_path.display().to_string();
    let file_path_abs = txtpp_file.join_base(base);
    let working_directory = get_parent(file_path_abs.as_ref())?;
    shell::run_shell(shell, arg, &working_directory, &file_path, directive_index)
}

fn get_parent(path: &Path) -> io::Result<PathBuf> {
    match path.parent() {
        Some(p) => Ok(p.to_path_buf()),
        None => Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Could not get the parent directory for {}", path.display()),
        )
        .into()),
    }
}

fn include_directive(base: &BasePath, txtpp_file: &BaseRelPath, arg: &str) -> io::Result<String> {
    let txtpp_file_path: &Path = txtpp_file.as_ref();
    let file_path = txtpp_file_path.display().to_string();
    let file_path_abs = txtpp_file.join_base(base);
    let working_directory = get_parent(file_path_abs.as_ref())?;
    let file_path = working_directory.join(arg);
    fs::read_to_string(file_path)
}

enum DepState {
    Pending,
    Done,
}

fn do_preprocess_singlethread(
    shell: &Shell,
    base: &BasePath,
    is_checking: bool,
    txtpp_file: &BaseRelPath,
    deps_stack: &mut Vec<BaseRelPath>,
    cache: &mut HashMap<BaseRelPath, DepState>,
) -> PrepResult {
    // check file not in cache
    if let Some(state) = cache.get(txtpp_file) {
        match state {
            DepState::Pending => return Err(PrepError::CircularDep(deps_stack.clone())),
            DepState::Done => return Ok(()),
        }
    }
    cache.insert(txtpp_file.clone(), DepState::Pending);

    let (mut r, mut w, le) = make_txtpp_context(base, txtpp_file, is_checking)?;
    let mut directive = directive::Result::NotDirective;
    let mut directive_index = 0;
    // read txtpp file line by line
    for line in r.lines() {
        let line = line?;
        // detect directive
        let (next_directive, output) = match directive {
            directive::Result::NotDirective => detect_directive(line, le),
            directive::Result::Directive(d) => {
                let output = match d.directive_type {
                    DirectiveType::Empty => {
                        // do nothing (consume the line)
                        None
                    }
                    DirectiveType::Include => {
                        // check if get_file_txtpp is Some
                        if let Some(x) = get_file_txtpp(base, &d.arg.clone().into()) {
                            deps_stack.push(x.clone());
                            do_preprocess_singlethread(
                                shell,
                                base,
                                is_checking,
                                &x,
                                deps_stack,
                                cache,
                            )?;
                            deps_stack.pop();
                        }
                        Some(include_directive(base, txtpp_file, &d.arg)?)
                    }
                    DirectiveType::Run => Some(run_directive(
                        shell,
                        base,
                        txtpp_file,
                        &d.arg,
                        directive_index,
                    )?),
                };
                directive_index += 1;
                let output = if let Some(x) = output {
                    Some(d.format_output(&x, le)?)
                } else {
                    None
                };
                (directive::Result::NotDirective, output)
            }
            directive::Result::NeedNextLine(d) => (d.addln(&line), None),
        };

        // write the file
        if let Some(output) = output {
            w.write(&output)?;
        }
        directive = next_directive;
    }
    w.done()?;
    if is_checking {
        // check if the thing exist and is the same
        verify(txtpp_file, base, w)?;
    }
    cache.insert(txtpp_file.clone(), DepState::Done);
    Ok(())
}
