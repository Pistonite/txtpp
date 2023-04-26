//! Utilities for running shell commands

use super::path::AbsPath;
use error_stack::{IntoReport, Report, Result};
use std::error;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::process::Command;
use which::which;

pub const TXTPP_FILE: &str = "TXTPP_FILE";

/// Error related to shell
#[derive(Debug)]
pub enum ShellError {
    ResolveError,
    ExecuteError,
}

impl std::fmt::Display for ShellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellError::ResolveError => write!(f, "Error resolving shell executable"),
            ShellError::ExecuteError => write!(f, "Error executing shell"),
        }
    }
}

impl error::Error for ShellError {}

/// Representation of a resolved shell command like `sh -c` or `cmd /C`
/// that takes a command as argument.
#[derive(Debug)]
pub struct Shell {
    /// The shell executable
    exe: String,
    /// The arguments of the shell executable (for example, `-c`)
    args: Vec<String>,
}

impl Display for Shell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.exe, self.args.join(" "))
    }
}

impl Shell {
    #[cfg(windows)]
    fn default() -> Result<Self, ShellError> {
        Self::new("powershell -c").or_else(|_| Self::new("cmd /C"))
    }
    #[cfg(not(windows))]
    fn default() -> Result<Self, ShellError> {
        Self::new("sh -c")
    }
    /// Create a new shell from the given command
    pub fn new(cmd: &str) -> Result<Self, ShellError> {
        // split the command into the executable and the arguments
        let mut args = cmd.split_whitespace();
        let exe = match args.next() {
            None => return Self::default(),
            Some(exe) => exe,
        };
        let exe = resolve_shell(exe)?.to_string();
        let args = args.map(String::from).collect::<Vec<_>>();

        // Resolve the absolute path of the shell executable
        Ok(Self { exe, args })
    }

    /// Run the shell with the given argument in the directory. Return the stdout.
    pub fn run<P>(&self, command: &str, work_dir: &P, file: &str) -> Result<String, ShellError>
    where
        P: AsRef<Path>,
    {
        log::debug!("shell command `{command}`");
        let result = Command::new(&self.exe)
            .current_dir(work_dir)
            .args(&self.args)
            .arg(command)
            .env(TXTPP_FILE, file)
            .output()
            .into_report()
            .map_err(|e| {
                e.change_context(ShellError::ExecuteError)
                    .attach_printable(format!(
                        "Failed to execute `{}` with shell `{}`",
                        command, self
                    ))
            })?;
        if result.status.success() {
            let output = String::from_utf8_lossy(&result.stdout).to_string();
            log::debug!("shell output `{output}`");
            Ok(output)
        } else {
            let exit_code = match result.status.code() {
                Some(code) => code.to_string(),
                None => "unknown".to_string(),
            };
            Err(
                Report::new(ShellError::ExecuteError).attach_printable(format!(
                    "Subcommand `{}` failed with exit code {}: {}",
                    command,
                    exit_code,
                    String::from_utf8_lossy(&result.stderr)
                )),
            )
        }
    }
}

fn resolve_shell(exe: &str) -> Result<AbsPath, ShellError> {
    let p = which(exe).unwrap_or_else(|_| Path::new(exe).to_path_buf());

    let p = p.canonicalize().into_report().map_err(|e| {
        e.change_context(ShellError::ResolveError)
            .attach_printable(format!(
                "could not resolve shell executable: {}",
                p.display()
            ))
    })?;

    let path = p.display().to_string();

    AbsPath::create_base(p).map_err(|e| {
        e.change_context(ShellError::ResolveError)
            .attach_printable(format!(
                "could not convert shell executable to absolute path: {}",
                path
            ))
    })
}
