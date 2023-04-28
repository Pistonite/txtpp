use clap::{Args, Parser, Subcommand};
use std::process::ExitCode;
use txtpp::{txtpp, Config, Mode, Verbosity};

/// txtpp CLI
///
/// See https://github.com/iTNTPiston/txtpp for more info
#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    subcommand: Option<Command>,
    #[command(flatten)]
    flags: Flags,
    #[command(flatten)]
    shell: ShellFlags,
}

impl Cli {
    fn apply_to(&self, config: &mut Config) {
        match &self.subcommand {
            Some(subcommand) => subcommand.apply_to(config),
            None => {
                config.mode = Mode::Build;
                self.flags.apply_to(config);
                self.shell.apply_to(config);
            }
        }
    }
}

#[derive(Debug, Clone, Subcommand)]
enum Command {
    /// Clean output files
    ///
    /// See https://docs.rs/txtpp/latest/txtpp/enum.Mode.html#variant.Clean for more details
    Clean {
        #[command(flatten)]
        flags: Flags,
    },
    /// Verify that files generated by txtpp are up to date
    ///
    /// See https://docs.rs/txtpp/latest/txtpp/enum.Mode.html#variant.Verify for more details
    Verify {
        #[command(flatten)]
        flags: Flags,
        #[command(flatten)]
        shell: ShellFlags,
    },
}

impl Command {
    fn apply_to(&self, config: &mut Config) {
        match self {
            Command::Clean { flags } => {
                config.mode = Mode::Clean;
                flags.apply_to(config);
            }
            Command::Verify { flags, shell } => {
                config.mode = Mode::Verify;
                flags.apply_to(config);
                shell.apply_to(config);
            }
        }
    }
}

#[derive(Debug, Clone, Args)]
struct Flags {
    /// Show no output.
    ///
    /// Errors will still be printed.
    #[arg(short, long)]
    quiet: bool,

    /// Show more output
    #[arg(short, long, conflicts_with = "quiet")]
    verbose: bool,

    /// If subdirectories should be recursively scanned for input files.
    ///
    /// By default (recursive=false), only directories specified in the input will be scanned.
    /// Newly discovered subdirectories will be ignored.
    #[arg(short, long)]
    recursive: bool,

    /// Specify the number of worker threads
    #[arg(short = 'j', long, default_value = "4")]
    threads: usize,

    /// Input files and/or directories
    ///
    /// Either the `.txtpp` input file or the corresponding output file should be specified.
    /// If a directory is specified, all `.txtpp` files in the directory will be processed.
    /// Subdirectories will not be processed unless `-r/--recursive` is specified.
    ///
    /// The current directory is used if no input is specified.
    #[arg(default_value = ".")]
    inputs: Vec<String>,
}

impl Flags {
    fn apply_to(&self, config: &mut Config) {
        if self.quiet {
            config.verbosity = Verbosity::Quiet;
        } else if self.verbose {
            config.verbosity = Verbosity::Verbose;
        }
        config.recursive = self.recursive;
        config.num_threads = self.threads;
        config.inputs = self.inputs.clone();
    }
}

#[derive(Debug, Clone, Args)]
struct ShellFlags {
    /// The shell command to use
    ///
    /// This should be a command that takes one argument, which is the command to run.
    /// For example `bash -c`. The string will be split on whitespace. The first segment
    /// will be the executable, and the rest will be arguments.
    ///
    /// If a shell is not specified, the platform-specific default shell will be used,
    /// which is `sh -c` on non-Windows. PowerShell is used on Windows with CMD as a fallback.
    /// See https://github.com/iTNTPiston/txtpp for the default PowerShell flags used.
    #[arg(short, long, default_value = "")]
    shell: String,
}

impl ShellFlags {
    fn apply_to(&self, config: &mut Config) {
        config.shell_cmd = self.shell.clone();
    }
}

fn main() -> ExitCode {
    eprintln!("This is a preview version and may not have all features implemented.");
    env_logger::init();
    let args = Cli::parse();

    log::debug!("{:?}", args);
    let mut config = Config::default();
    args.apply_to(&mut config);

    match txtpp(config) {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
