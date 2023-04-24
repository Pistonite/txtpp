use crate::fs::{PathError, AbsPath, Directory, Shell, TxtppPath};
use error_stack::{IntoReport, Report, Result};
use termcolor::Color;
use threadpool::ThreadPool;
use std::error;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use threadpool::Builder;
use super::verbs;
use super::{PreprocessResult, PreprocessError};
use super::dependency::DepManager;
use super::preprocess::do_preprocess;
use log;
use crate::progress::Progress;

#[derive(Debug)]
pub struct Config {
    pub shell_cmd: String,
    pub inputs: Vec<String>,
    pub recursive: bool,
    pub num_threads: usize,
    pub verify: bool,
}

#[derive(Debug)]
pub struct ExecuteError;

impl fmt::Display for ExecuteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error executing txtpp")
    }
}

impl error::Error for ExecuteError {}

pub fn execute(config: Config) -> Result<(), ExecuteError> {
    log::info!("starting txtpp");
    log::info!("config is: {:?}", config);
    let base_abs_path = AbsPath::try_from(PathBuf::from(".")).map_err(|e| {
        e.change_context(ExecuteError).attach_printable("cannot get current directory")
            
    })?;

    let shell = Shell::new(&config.shell_cmd).map_err(|e| {
        e.change_context(ExecuteError).attach_printable(format!(
            "cannot parse shell command: {cmd}",
            cmd = config.shell_cmd
        ))
        
    })?;

    // the remaining files and directories to process
    let inputs: Directory = resolve_inputs(&config.inputs, &base_abs_path)?;
    log::info!("inputs are: {:?}", inputs);

    let threadpool = Builder::new().num_threads(config.num_threads).build();


    let result = execute_loop(&threadpool, inputs, shell, config);
    threadpool.join();

    log::info!("txtpp done");
    
    result
}

fn execute_directory(directory: Directory, threadpool: &ThreadPool, send: &mpsc::Sender<TaskResult>, shell: Arc<Shell>, config: Arc<Config>, progress: &mut Progress) {
    // start processing input files
    execute_files(directory.files.into_iter(), threadpool, send, shell, config.clone(), progress);

    for dir in directory.subdirs {
        let _ = progress.print_status(verbs::SCANNING, &dir.to_string(), Color::Yellow);
        let send = send.clone();
        let config = config.clone();
        log::info!("scanning directory: {dir}");
        threadpool.execute(move || {
            let result = scan_dir(&dir, config.recursive);
            let _ = send.send(TaskResult::ScanDir(result));
        });
    }
    // start processing input directories
}

fn execute_files(files: impl Iterator<Item=AbsPath>, threadpool: &ThreadPool, send: &mpsc::Sender<TaskResult>, shell: Arc<Shell>, config: Arc<Config>, progress: &mut Progress) {
    for file in files {
        let _ = progress.print_status(verbs::PROCESSING, &file.to_string(), Color::Green);
        let send = send.clone();
        let shell = shell.clone();
        let config = config.clone();
        log::info!("processing file: {file}");
        threadpool.execute(move || {
            let result = do_preprocess(&shell, &file, config.verify, true);
            let _ = send.send(TaskResult::Preprocess(result));
        });
    }
}

fn execute_loop(threadpool: &ThreadPool, inputs: Directory, shell: Shell, config: Config) -> Result<(), ExecuteError> {
    let shell = Arc::new(shell);
    let config = Arc::new(config);
    let mut dep_mgr = DepManager::new();
    let mut total_count = inputs.files.len() + inputs.subdirs.len();
    
    let mut progress = Progress::new();
    let (send, recv) = mpsc::channel();
    execute_directory(inputs, &threadpool, &send, shell.clone(), config.clone(), &mut progress);
    let mut done_count = 0;
    let mut file_count = 0;
    loop {
        let data = match recv.try_recv() {
            Ok(data) => data,
            Err(TryRecvError::Empty) => {
                if done_count == total_count {
                    break;
                }
                // no data available, wait for a bit
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            },
            Err(TryRecvError::Disconnected) => {
                // all senders are dropped, no more data will be available
                return Err(Report::new(ExecuteError).attach_printable("workers are disconnected unexpectedly."));
            }
        };

        match data {
            TaskResult::ScanDir(result) => {
                log::info!("scanning directory done");
                let directory = result.map_err(|e| {
                    e.change_context(ExecuteError).attach_printable("cannot scan directory")
                        
                })?;
                total_count += directory.files.len() + directory.subdirs.len();
                execute_directory(directory, &threadpool, &send, shell.clone(), config.clone(), &mut progress);
            }
            TaskResult::Preprocess(result) => {
                let preprocess_result = result.map_err(|e| {
                    e.change_context(ExecuteError).attach_printable("cannot preprocess file")
                        
                })?;
                match preprocess_result {
                    PreprocessResult::HasDeps(input, deps) => {
                        log::info!("file {input} has dependencies: {deps:?}");
                        dep_mgr.add_dependency(&input, &deps);
                    },
                    PreprocessResult::Ok(input) => {
                        log::info!("file {input} done");
                        file_count += 1;
                        let files = dep_mgr.notify_finish(&input).into_iter();
                        total_count += files.len();
                        execute_files(files, &threadpool, &send, shell.clone(), config.clone(), &mut progress)
                    }
                }
            }
        }

        
        done_count += 1;
        log::info!("progress: {}/{}", done_count, total_count);
        let _ = progress.update_progress(done_count, total_count);

    }

    let _ = progress.print_status(verbs::SCANNED, &format!("{total_count} path(s)."), Color::Yellow);
        let _ = progress.print_status(verbs::DONE, &format!("{file_count} file(s)"), Color::Green);

    Ok(())

}

fn resolve_inputs(inputs: &[String], base_abs_path: &AbsPath) -> Result<Directory, ExecuteError> {
    let mut directory = Directory::new();
    for input in inputs {
        let input_path = base_abs_path.as_path_buf().join(input);
        if input_path.is_dir() {
            // if input is directory, add to the directories to scan
            let abs_path = AbsPath::try_from(input_path).map_err(|e| {
                e.change_context(ExecuteError).attach_printable(format!("cannot resolve input: {input}"))
                    
            })?;
            directory.subdirs.push(abs_path);
        } else if !input_path.is_txtpp_file() {
            // input is a file but not a txtpp file
            // not that input file doesn't have to exist
            if let Some(input_path) = input_path.get_txtpp_file() {
                let abs_path = AbsPath::try_from(input_path).map_err(|e| {
                    e.change_context(ExecuteError).attach_printable(format!("cannot resolve input: {input}"))
                        
                })?;
                directory.files.push(abs_path);
            }
        } else {
            // input is txtpp file. it must exist
            let abs_path = AbsPath::try_from(input_path).map_err(|e| {
                e.change_context(ExecuteError).attach_printable(format!("cannot resolve input: {input}"))
                    
            })?;
            directory.files.push(abs_path);
        }
    }
    Ok(directory)
}

enum TaskResult {
    ScanDir(Result<Directory, PathError>),
    Preprocess(Result<PreprocessResult, PreprocessError>),
}

fn scan_dir(dir: &AbsPath, recursive: bool) -> Result<Directory, PathError> {
    let dir_path = dir.as_path_buf();
    let entries = dir_path.read_dir().into_report().map_err(|e| {
        e.change_context(PathError::from(dir_path)).attach_printable("failed to read directory")
    })?;

    let mut directory = Directory::new();

    for entry in entries {
        let entry = entry.into_report().map_err(|e| {
            e.change_context(PathError::from(dir_path)).attach_printable("failed to read directory")
        })?;
        let path = entry.path();

        if path.is_file() {
            if path.is_txtpp_file() {
                let path_abs = AbsPath::try_from(path.clone())?;
                directory.files.push(path_abs);
            }

        } else if path.is_dir() && recursive {
            let path_abs = AbsPath::try_from(path.clone())?;
            directory.subdirs.push(path_abs);
        }
    }

    Ok(directory)
}