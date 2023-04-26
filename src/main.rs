use clap::{Args, Parser, Subcommand};
use std::process::ExitCode;
use txtpp::{txtpp, Config, Mode, Verbosity};

/// TODO doc needed CLI
///
/// TODO long doc needed CLI
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
    /// TODO doc needed clean subcommand
    Clean {
        #[command(flatten)]
        flags: Flags,
    },
    /// TODO doc needed verify subcommand
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
    /// TODO doc needed quiet
    #[arg(short, long)]
    quiet: bool,

    /// TODO doc needed verbose
    #[arg(short, long, conflicts_with = "quiet")]
    verbose: bool,

    /// TODO doc needed recursive
    #[arg(short, long)]
    recursive: bool,

    /// TODO doc needed threads
    #[arg(short = 'j', long, default_value = "4")]
    threads: usize,

    /// TODO doc needed inputs
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
    /// TODO doc needed shell
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
