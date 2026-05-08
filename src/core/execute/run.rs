use std::collections::HashSet;
use std::sync::Arc;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::time::Instant;

use cu::pre::*;
use termcolor::Color;
use threadpool::Builder;
use threadpool::ThreadPool;

use crate::core::execute::{PpResult, preprocess, resolve_inputs, scan_dir};
use crate::core::{Config, DepManager, Progress, print_dep_map, verbs};
use crate::error::PpError;
use crate::fs::{AbsPath, Directory, Shell};

/// Run txtpp with the given config
///
/// This is the main entry point for txtpp. It takes a [`Config`] and runs txtpp.
/// If an error occurs, it will be printed to stderr.
///
/// If you want to retrieve the error report without printing it, use [`Txtpp::run`].
pub fn txtpp(config: Config) -> cu::Result<()> {
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
    pub fn run(config: Config) -> cu::Result<()> {
        cu::debug!("creating txtpp");
        cu::debug!("using config: {:?}", config);

        let shell = cu::check!(
            Shell::new(&config.shell_cmd),
            "cannot parse shell command: {}",
            config.shell_cmd
        )?;
        let shell = Arc::new(shell);

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

    fn run_internal(&mut self) -> cu::Result<()> {
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

        let base_abs_path = cu::check!(
            AbsPath::create_base(self.config.base_dir.clone()),
            "cannot resolve base directory"
        )?;
        let inputs: Directory = cu::check!(
            resolve_inputs(&self.config.inputs, &base_abs_path),
            "cannot resolve inputs"
        )?;
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
                    cu::bail!("workers are disconnected unexpectedly");
                }
            };

            let _ = self.progress.add_done(1);

            match data {
                TaskResult::ScanDir(result) => {
                    cu::debug!("scanning directory done");
                    let directory = cu::check!(result, "cannot can directory")?;
                    let _ = self.progress.add_total(directory.subdirs.len());
                    for file in directory.files {
                        self.execute_file(file, true)?;
                    }
                    for dir in directory.subdirs {
                        self.execute_directory(dir, self.config.recursive);
                    }
                }
                TaskResult::Preprocess(result) => {
                    let preprocess_result = match result {
                        Ok(x) => x,
                        Err(e) => {
                            self.progress.add_done_quiet(1);
                            cu::bail!("{e:?}");
                        }
                    };
                    match preprocess_result {
                        PpResult::HasDeps(input, deps) => {
                            cu::debug!("file {input} has dependencies: {deps:?}");
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
                            cu::debug!("file {input} done");
                            let file_target =
                                cu::check!(input.trim_txtpp(), "cannot trim txtpp extension")?;
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
            // TODO: --rewrite: beautify
            cu::bail!("circular dependencies found: {}", print_dep_map(&remaining));
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
        cu::debug!("scanning directory: {dir}");
        self.threadpool.execute(move || {
            let result = scan_dir(&dir, recursive);
            send.send(TaskResult::ScanDir(result))
                .expect("cannot send result")
        });
    }

    fn execute_file(&mut self, file: AbsPath, is_first_pass: bool) -> cu::Result<()> {
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
        let file_target = cu::check!(file.trim_txtpp(), "cannot trim txtpp extension")?;
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
        cu::debug!("processing file: {file}");
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
        cu::debug!("cleaning up txtpp");
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
        cu::debug!("txtpp destroyed");
        // the channel will be dropped
    }
}

enum TaskResult {
    ScanDir(cu::Result<Directory>),
    Preprocess(error_stack::Result<PpResult, PpError>),
}
