use crate::constants::{TXTPP_DIRECTIVE_INDEX, TXTPP_FILE};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use which::which;

pub struct Shell {
    /// The shell executable
    pub exe: String,
    /// The arguments of the shell executable
    pub args: Vec<String>,
}

/// Resolve the absolute path of the shell executable
pub fn resolve_shell(exe: &str) -> io::Result<String> {
    let which_result = which(exe);
    if let Ok(p) = which_result {
        return Ok(p.display().to_string());
    }
    let p = Path::new(".").join(exe);
    if !p.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Could not resolve shell executable: {}", exe),
        ));
    }
    Ok(p.canonicalize()?.display().to_string())
}

/// Run the shell with the given argument in the directory. Return the stdout.
pub fn run_shell(
    shell: &Shell,
    arg: &str,
    working_directory: &Path,
    file: &str,
    directive_index: i32,
) -> io::Result<String> {
    let result = Command::new(&shell.exe)
        .current_dir(working_directory)
        .args(&shell.args)
        .arg(arg)
        .env(TXTPP_FILE, file)
        .env(TXTPP_DIRECTIVE_INDEX, format!("{}", directive_index))
        .output()?;
    if result.status.success() {
        Ok(String::from_utf8_lossy(&result.stdout).to_string())
    } else {
        let exit_code = match result.status.code() {
            Some(code) => code.to_string(),
            None => "unknown".to_string(),
        };
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Subcommand `{}` failed with exit code {} using shell `{}` running in directory `{}`:\n{}",
                arg,
                exit_code,
                shell.exe,
                working_directory.display(),
                String::from_utf8_lossy(&result.stderr).to_string()
            ),
        ))
    }
}
