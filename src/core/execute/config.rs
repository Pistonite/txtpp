use crate::core::verbs;
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
    /// If the output files should have trailing newline
    pub trailing_newline: bool,
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
    /// - Output files have trailing newline
    fn default() -> Self {
        Self {
            base_dir: PathBuf::from("."),
            shell_cmd: "".to_string(),
            inputs: vec![".".to_string()],
            recursive: false,
            num_threads: 4,
            mode: Mode::Build,
            verbosity: Verbosity::Normal,
            trailing_newline: true,
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
    /// Build output files
    ///
    /// This is the default mode if no subcommand is specified from the command line.
    /// For every `.txtpp` file found from the inputs, it will build the output file and any temporary file
    /// the `.txtpp` may produce. Dependencies in `include` directives will be built automatically as well, even if
    /// not specified in the inputs.
    Build,
    /// Delete output files
    ///
    /// Remove the output file and any temporary file the `.txtpp` input files may produce.
    /// Dependency is not automatically cleaned if they are not specified in the inputs.
    Clean,
    /// Verify output files are up to date
    ///
    /// In this mode, the output files will be compared against output from a fresh run.
    /// The run will fail if any output is different from the fresh output. Note that the temporary files
    /// are not compared, and the temporary files may be rebuilt in the process in order to generate the fresh output.
    Verify,
}

impl Mode {
    pub fn processing_verb(&self) -> &'static str {
        match self {
            Self::Build => verbs::PROCESSING,
            Self::Clean => verbs::CLEANING,
            Self::Verify => verbs::VERIFYING,
        }
    }
}
