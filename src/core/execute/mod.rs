use crate::core::{verbs, DepManager, Progress, print_dep_map};
use crate::fs::{AbsPath, Directory, PathError, Shell};
use error_stack::{Report, Result};
use log;
use std::collections::HashSet;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::sync::Arc;
use termcolor::Color;
use threadpool::Builder;
use threadpool::ThreadPool;

mod config;
pub use config::*;

mod error;
use error::ExecuteError;
mod preprocess;
pub use preprocess::PreprocessError;
use preprocess::{do_preprocess, PreprocessResult};
mod resolve_inputs;
use resolve_inputs::resolve_inputs;
mod scan_dir;
use scan_dir::scan_dir;

/// Run txtpp with the given config
///
/// This is the main entry point for txtpp. It takes a [`Config`] and runs txtpp.
/// If an error occurs, it will be printed to stderr and the function will return [`Err`].
///
/// If you want to retrieve the error object instead of printing it, use [`Txtpp::run`].
pub fn txtpp(config: Config) -> std::result::Result<(), ()> {
    match Txtpp::run(config) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("{:?}", e);
            Err(())
        }
    }
}

/// The runtime state when executing txtpp
#[derive(Debug)]
pub struct Txtpp {
    /// The Config
    config: Config,
    /// The Shell
    shell: Arc<Shell>,
    /// The Progress reporter
    progress: Progress,
    /// The ThreadPool
    threadpool: ThreadPool,
    /// The Sender for workers to send results back
    send: mpsc::Sender<TaskResult>,
    /// The Receiver for the main thread to receive results
    recv: mpsc::Receiver<TaskResult>,
    /// Files in the build system
    /// 
    /// This is to track we don't unnecessarily process the same file twice in the first pass
    files: HashSet<AbsPath>,
    /// The verb for the current processing mode (e.g. "Txtpping", "Cleaning", "Verifying")
    process_verb: &'static str
}

impl Txtpp {
    /// Internal run function
    ///
    /// This is what [`txtpp`] calls internally. The difference is that this function
    /// returns the error instead of printing it.
    pub fn run(config: Config) -> Result<(), ExecuteError> {
        log::info!("creating txtpp");
        log::debug!("using config: {:?}", config);

        let shell = Arc::new(Shell::new(&config.shell_cmd).map_err(|e| {
            e.change_context(ExecuteError).attach_printable(format!(
                "cannot parse shell command: {cmd}",
                cmd = config.shell_cmd
            ))
        })?);

        let progress = Progress::new(config.verbosity.clone());

        let threadpool = Builder::new().num_threads(config.num_threads).build();
        let (send, recv) = mpsc::channel();

        let runtime = Self {
            process_verb: config.mode.processing_verb(),
            config,
            shell,
            progress,
            threadpool,
            send,
            recv,
            files: HashSet::new(),
        };

        runtime.run_internal()
    }

    fn run_internal(mut self) -> Result<(), ExecuteError> {
        let _ =
            self.progress
                .print_status(verbs::USING, &self.shell.to_string(), Color::Yellow, false);
        let _ = self.progress.print_status(
            verbs::USING,
            &format!("{} thread(s)", self.config.num_threads),
            Color::Yellow,
            false,
        );

        let base_abs_path = AbsPath::create_base(self.config.base_dir.clone()).map_err(|e| {
            e.change_context(ExecuteError)
                .attach_printable("cannot resolve base directory")
        })?;
        let inputs: Directory =
            resolve_inputs(&self.config.inputs, &base_abs_path).map_err(|e| {
                e.change_context(ExecuteError)
                    .attach_printable("cannot resolve inputs")
            })?;
        let mut dep_mgr = DepManager::new();
        let mut file_count = 0;
        let _ = self
            .progress
            .add_total(inputs.subdirs.len());

        // schedule input files
        for file in inputs.files {
            self.execute_file(file.clone(), true);
        }
        // schedule input directories
        for dir in inputs.subdirs {
            self.execute_directory(dir.clone(), self.config.recursive);
        }

        loop {
            let data = match self.recv.try_recv() {
                Ok(data) => data,
                Err(TryRecvError::Empty) => {
                    if self.progress.is_done() {
                        break;
                    }
                    // no data available, wait for a bit
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                }
                Err(TryRecvError::Disconnected) => {
                    // workers are disconnected unexpectedly
                    return Err(Report::new(ExecuteError)
                        .attach_printable("workers are disconnected unexpectedly."));
                }
            };

            match data {
                TaskResult::ScanDir(result) => {
                    log::info!("scanning directory done");
                    let directory = result.map_err(|e| {
                        let _ = self
                            .progress
                            .print_status(verbs::FAILED, "", Color::Red, false);
                        self.progress.add_done_quiet(1);
                        e.change_context(ExecuteError)
                            .attach_printable("cannot scan directory")
                    })?;
                    let _ = self
                        .progress
                        .add_total(directory.subdirs.len());
                    for file in directory.files {
                        self.execute_file(file, true);
                    }
                    for dir in directory.subdirs {
                        self.execute_directory(dir, self.config.recursive);
                    }
                }
                TaskResult::Preprocess(result) => {
                    let preprocess_result = result.map_err(|e| {
                        let _ = self
                            .progress
                            .print_status(verbs::FAILED, "", Color::Red, false);
                        self.progress.add_done_quiet(1);
                        e.change_context(ExecuteError)
                    })?;
                    match preprocess_result {
                        PreprocessResult::HasDeps(input, deps) => {
                            log::info!("file {input} has dependencies: {deps:?}");
                            dep_mgr.add_dependency(&input, &deps);
                            // schedule the dependencies
                            for dep in deps {
                                self.execute_file(dep, true);
                            }
                        }
                        PreprocessResult::Ok(input) => {
                            log::info!("file {input} done");
                            file_count += 1;
                            let files = dep_mgr.notify_finish(&input);
                            for file in files {
                                self.execute_file(file, false);
                            }
                        }
                    }
                }
            }

            let _ = self.progress.add_done(1);
        }

        // make sure all dependencies are processed
        let remaining = dep_mgr.take_remaining();
        if !remaining.is_empty() {
            let _ = self
                .progress
                .print_status(verbs::FAILED, "", Color::Red, false);
            return Err(Report::new(ExecuteError)
                .attach_printable("Circular dependencies are found:")
                .attach_printable(print_dep_map(&remaining)));
        }

        let _ = self.progress.print_status(
            verbs::SCANNED,
            &format!("{} path(s).", self.progress.total_count),
            Color::Yellow,
            false,
        );
        let _ = self.progress.print_status(
            verbs::DONE,
            &format!("{file_count} file(s)"),
            Color::Green,
            false,
        );

        Ok(())
    }

    fn execute_directory(&mut self, dir: AbsPath, recursive: bool) {
        let _ = self
            .progress
            .print_status(verbs::SCANNING, &dir.to_string(), Color::Yellow, true);
        let send = self.send.clone();
        log::info!("scanning directory: {dir}");
        self.threadpool.execute(move || {
            let result = scan_dir(&dir, recursive);
            send.send(TaskResult::ScanDir(result))
                .expect("cannot send result")
        });
    }

    fn execute_file(&mut self, file: AbsPath, is_first_pass: bool) {
        if is_first_pass {
            // There could be duplicate inputs for multiple reasons:
            // 1. The user specified the same input multiple times.
            // 2. The input is both specified by user and discovered as dependency
            // Therefore, we need to prevent processing the same file multiple times in the first pass.
            if !self.files.insert(file.clone()) {
                return;
            }
        }
        let _ = self.progress.add_total(1);
        let _ =
            self.progress
                .print_status(self.process_verb, &file.to_string(), Color::Green, false);
        let send = self.send.clone();
        let shell = self.shell.clone();
        let mode = self.config.mode.clone();
        log::info!("processing file: {file}");
        self.threadpool.execute(move || {
            let result = do_preprocess(&shell, &file, mode, is_first_pass);
            send.send(TaskResult::Preprocess(result))
                .expect("cannot send result")
        });
    }
}

impl Drop for Txtpp {
    fn drop(&mut self) {
        log::info!("cleaning up txtpp");
        self.threadpool.join();
        // wait for all workers to finish sending their last results, which we will ignore
        loop {
            match self.recv.try_recv() {
                Ok(_) => {
                    self.progress.add_done_quiet(1);
                }
                Err(TryRecvError::Empty) => {
                    if self.progress.is_done() {
                        break;
                    }
                    // no data available, wait for a bit
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                }
                Err(TryRecvError::Disconnected) => {
                    break;
                }
            }
        }
        log::info!("txtpp destroyed");
        // the channel will be dropped
    }
}

enum TaskResult {
    ScanDir(Result<Directory, PathError>),
    Preprocess(Result<PreprocessResult, PreprocessError>),
}
