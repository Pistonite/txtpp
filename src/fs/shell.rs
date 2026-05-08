//! Utilities for running shell commands

use std::fmt::{Display, Formatter};
use std::path::Path;
use std::process::Command;

use cu::pre::*;

use super::path::AbsPath;

/// Name of the environment variable set for subcommands for the current file being processed
pub const TXTPP_FILE: &str = "TXTPP_FILE";

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
    fn try_default() -> cu::Result<Self> {
        Self::new("pwsh -NonInteractive -NoProfile -Command")
            .or_else(|_| Self::new("powershell -NonInteractive -NoProfile -Command"))
            .or_else(|_| Self::new("cmd /C"))
    }
    #[cfg(not(windows))]
    fn try_default() -> cu::Result<Self> {
        Self::new("sh -c")
    }
    /// Create a new shell from the given command
    pub fn new(cmd: &str) -> cu::Result<Self> {
        // split the command into the executable and the arguments
        let mut args = cmd.split_whitespace();
        let exe = match args.next() {
            None => return Self::try_default(),
            Some(exe) => exe,
        };
        let exe = resolve_shell(exe)?.to_string();
        let args = args.map(String::from).collect::<Vec<_>>();

        // Resolve the absolute path of the shell executable
        Ok(Self { exe, args })
    }

    /// Run the shell with the given argument in the directory. Return the stdout.
    pub fn run(&self, command: &str, work_dir: &AbsPath, file: &str) -> cu::Result<String> {
        // TODO: --rewrite: this should just be cu::command
        cu::debug!("shell command `{command}`");
        let result = Command::new(&self.exe)
            .current_dir(work_dir.to_string())
            .args(&self.args)
            .arg(command)
            .env(TXTPP_FILE, file)
            .output();
        let result = cu::check!(
            result,
            "failed to execute `{}` with shell `{}`",
            command,
            self
        )?;
        if result.status.success() {
            let output = String::from_utf8_lossy(&result.stdout).to_string();
            cu::debug!("shell output `{output}`");
            Ok(output)
        } else {
            let exit_code = match result.status.code() {
                Some(code) => code.to_string(),
                None => "unknown".to_string(),
            };
            cu::error!("{}", String::from_utf8_lossy(&result.stderr));
            cu::bail!("subcommand `{command}` failed with exit code {exit_code}");
        }
    }
}

fn resolve_shell(exe: &str) -> cu::Result<AbsPath> {
    // TODO: --rewrite: this should just be cu::which, then AbsPath
    let p = cu::which(exe).unwrap_or_else(|_| Path::new(exe).to_path_buf());
    // this is actually bad for sym/hard links
    let p = cu::check!(
        p.canonicalize(),
        "could not resolve shell executable: '{}'",
        p.display()
    )?;
    let path = p.display().to_string();
    cu::check!(
        AbsPath::create_base(p),
        "could not convert shell executable to absolute: '{}'",
        path
    )
}
