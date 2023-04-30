use crate::core::{print_dep_map, verbs, DepManager, Progress};
use crate::error::{PathError, PpError, TxtppError};
use crate::fs::{AbsPath, Directory, Shell};
use error_stack::{Report, Result};
use std::collections::HashSet;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::sync::Arc;
use std::time::Instant;
use termcolor::Color;
use threadpool::Builder;
use threadpool::ThreadPool;

mod config;
pub use config::*;

mod pp;
use pp::{preprocess, PpResult};
mod resolve_inputs;
use resolve_inputs::resolve_inputs;
mod scan_dir;
use scan_dir::scan_dir;

/// Run txtpp with the given config
///
/// This is the main entry point for txtpp. It takes a [`Config`] and runs txtpp.
/// If an error occurs, it will be printed to stderr.
///
/// If you want to retrieve the error report without printing it, use [`Txtpp::run`].
pub fn txtpp(config: Config) -> Result<(), TxtppError> {
    match Txtpp::run(config) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("{:?}", e);
            Err(e)
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
}

impl Txtpp {
    /// Internal run function
    ///
    /// This is what [`txtpp`] calls internally. The difference is that this function
    /// returns the error instead of printing it.
    pub fn run(config: Config) -> Result<(), TxtppError> {
        log::info!("creating txtpp");
        log::debug!("using config: {:?}", config);

        let shell = Arc::new(Shell::new(&config.shell_cmd).map_err(|e| {
            e.change_context(TxtppError).attach_printable(format!(
                "cannot parse shell command: {cmd}",
                cmd = config.shell_cmd
            ))
        })?);

        let progress = Progress::new(config.verbosity.clone());

        let threadpool = Builder::new().num_threads(config.num_threads).build();
        let (send, recv) = mpsc::channel();

        let mut runtime = Self {
            config,
            shell,
            progress,
            threadpool,
            send,
            recv,
            files: HashSet::new(),
        };

        let result = runtime.run_internal();
        if result.is_err() {
            let _ = runtime
                .progress
                .print_status(verbs::FAILED, "", Color::Red, false);
            runtime.progress.has_error = true;
        }

        result
    }

    fn run_internal(&mut self) -> Result<(), TxtppError> {
        let start_time = Instant::now();
        let _ =
            self.progress
                .print_status(verbs::USING, &self.shell.to_string(), Color::Yellow, true);
        let _ = self.progress.print_status(
            verbs::USING,
            &format!("{} thread(s)", self.config.num_threads),
            Color::Yellow,
            true,
        );

        let base_abs_path = AbsPath::create_base(self.config.base_dir.clone()).map_err(|e| {
            e.change_context(TxtppError)
                .attach_printable("cannot resolve base directory")
        })?;
        let inputs: Directory =
            resolve_inputs(&self.config.inputs, &base_abs_path).map_err(|e| {
                e.change_context(TxtppError)
                    .attach_printable("cannot resolve inputs")
            })?;
        let mut dep_mgr = DepManager::new();
        let mut file_count = 0;
        let _ = self.progress.add_total(inputs.subdirs.len());

        // schedule input files
        for file in inputs.files {
            self.execute_file(file.clone(), true)?;
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
                    return Err(Report::new(TxtppError)
                        .attach_printable("workers are disconnected unexpectedly."));
                }
            };

            let _ = self.progress.add_done(1);

            match data {
                TaskResult::ScanDir(result) => {
                    log::info!("scanning directory done");
                    let directory = result.map_err(|e| {
                        self.progress.add_done_quiet(1);
                        e.change_context(TxtppError)
                            .attach_printable("cannot scan directory")
                    })?;
                    let _ = self.progress.add_total(directory.subdirs.len());
                    for file in directory.files {
                        self.execute_file(file, true)?;
                    }
                    for dir in directory.subdirs {
                        self.execute_directory(dir, self.config.recursive);
                    }
                }
                TaskResult::Preprocess(result) => {
                    let preprocess_result = result.map_err(|e| {
                        self.progress.add_done_quiet(1);
                        e.change_context(TxtppError)
                    })?;
                    match preprocess_result {
                        PpResult::HasDeps(input, deps) => {
                            log::info!("file {input} has dependencies: {deps:?}");
                            if dep_mgr.add_dependency(&input, &deps) {
                                // schedule the dependencies
                                for dep in deps {
                                    self.execute_file(dep, true)?;
                                }
                            } else {
                                // the dependencies are already done, shedule the file again
                                self.execute_file(input, false)?;
                            }
                        }
                        PpResult::Ok(input) => {
                            log::info!("file {input} done");
                            let file_target = input.trim_txtpp().map_err(|e| {
                                e.change_context(TxtppError)
                                    .attach_printable("cannot trim txtpp extension")
                            })?;
                            let _ = self.progress.print_status(
                                self.config.mode.processed_verb(),
                                &file_target,
                                Color::Green,
                                false,
                            );
                            file_count += 1;
                            let files = dep_mgr.notify_finish(&input);
                            for file in files {
                                self.execute_file(file, false)?;
                            }
                        }
                    }
                }
            }
        }

        // make sure all dependencies are processed
        let remaining = dep_mgr.take_remaining();
        if !remaining.is_empty() {
            return Err(Report::new(TxtppError)
                .attach_printable("Circular dependencies are found:")
                .attach_printable(print_dep_map(&remaining)));
        }

        let _ = self.progress.print_status(
            verbs::SCANNED,
            &format!("{} path(s)", self.progress.total_count),
            Color::Yellow,
            true,
        );

        let elapsed = start_time.elapsed();

        let _ = self.progress.print_status(
            verbs::DONE,
            &format!("{file_count} file(s) in {:.2}s", elapsed.as_secs_f32()),
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

    fn execute_file(&mut self, file: AbsPath, is_first_pass: bool) -> Result<(), TxtppError> {
        if is_first_pass {
            // There could be duplicate inputs for multiple reasons:
            // 1. The user specified the same input multiple times.
            // 2. The input is both specified by user and discovered as dependency
            // Therefore, we need to prevent processing the same file multiple times in the first pass.
            if !self.files.insert(file.clone()) {
                return Ok(());
            }
        }

        let _ = self.progress.add_total(1);
        let file_target = file.trim_txtpp().map_err(|e| {
            e.change_context(TxtppError)
                .attach_printable("cannot trim txtpp extension")
        })?;
        let _ = self.progress.print_status(
            self.config.mode.processing_verb(),
            &file_target,
            Color::Yellow,
            true,
        );
        let send = self.send.clone();
        let shell = self.shell.clone();
        let mode = self.config.mode.clone();
        let trailing_newline = self.config.trailing_newline;
        log::info!("processing file: {file}");
        self.threadpool.execute(move || {
            let result = preprocess(&shell, &file, mode, is_first_pass, trailing_newline);
            send.send(TaskResult::Preprocess(result))
                .expect("cannot send result")
        });
        Ok(())
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
                    if self.progress.is_done() || self.progress.has_error {
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
    Preprocess(Result<PpResult, PpError>),
}
