use std::path::PathBuf;

/// Config for running txtpp
/// 
/// Use this to configure txtpp when calling it from the library
/// # Example
/// ```
/// use txtpp::{Txtpp, Config, Verbosity};
/// 
/// // Use the default config
/// let mut cfg = Config::default();
/// // Change verbosity to verbose
/// cfg.verbosity = Verbosity::Verbose;
/// Txtpp::run(cfg).unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct Config {
    /// Base directory for resolving paths in input. This is usually the current directory.
    pub base_dir: PathBuf,
    /// The shell command to use. (e.g. `bash -c`). Empty string for platform-specific default shell
    pub shell_cmd: String,
    /// The input file/directories
    pub inputs: Vec<String>,
    /// Whether to recursively process directories
    pub recursive: bool,
    /// The number of threads to use
    pub num_threads: usize,
    /// The mode. See [`Mode]
    pub mode: Mode,
    /// The verbosity. See [`Verbosity`]
    pub verbosity: Verbosity,
}

impl Default for Config {
    /// Get the default config.
    /// 
    /// This means:
    /// - Running from the current directory
    /// - Using the platform-specific default shell
    /// - Processing the current directory
    /// - Not recursively processing directories
    /// - Using 4 threads
    /// - Building output files
    /// - Regular verbosity
    fn default() -> Self {
        Self {
            base_dir: PathBuf::from("."),
            shell_cmd: "".to_string(),
            inputs: vec![".".to_string()],
            recursive: false,
            num_threads: 4,
            mode: Mode::Build,
            verbosity: Verbosity::Normal,
        }
    }
}

/// The verbosity config options
#[derive(Debug, PartialEq, Clone)]
pub enum Verbosity {
    Quiet,
    Normal,
    Verbose,
}

/// The mode config options
#[derive(Debug, PartialEq, Clone)]
pub enum Mode {
    /// build output files
    Build,
    /// Delete output files
    Clean,
    /// Verify output files are the same as fresh output
    Verify
}