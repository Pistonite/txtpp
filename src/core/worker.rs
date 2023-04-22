use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use crate::path::{self, AbsPath, Directory};
use crate::{PreprocessError, PreprocessResult};
use error_stack::{IntoReport, Report, Result};



/// Task to be performed by a worker
pub enum Task {
    /// Scan directory and return list of txtpp files and directories
    ScanDir(PathBuf),
    /// Preprocess file and return result
    Preprocess(AbsPath),
}

pub enum TaskResult {
    ScanDir(Result<Directory, path::Error>),
    Preprocess(Result<PreprocessResult, PreprocessError>),
}

pub fn start_worker(
    id: usize,
    send: mpsc::Sender<(usize, TaskResult)>,
) -> (thread::JoinHandle<Option<TaskResult>>, mpsc::Sender<Task>) {
    let (ret_send, recv) = mpsc::channel();

    let join_handle = thread::spawn(move || worker_loop(id, send, recv));

    (join_handle, ret_send)
}

fn worker_loop(
    id: usize,
    send: mpsc::Sender<(usize, TaskResult)>,
    recv: mpsc::Receiver<Task>,
) -> Option<TaskResult> {
    loop {
        let task = match recv.recv() {
            Ok(t) => t,
            Err(_) => return None,
        };

        let result = match task {
            Task::ScanDir(path) => {
                let result = scan_dir(path);
                TaskResult::ScanDir(result)
            }
            Task::Preprocess(path) => {
                todo!();
                //let result = crate::preprocess(path);
                // TaskResult::Preprocess(result)
            }
        };

        match send.send((id, result)) {
            Ok(_) => {}
            Err(e) => return Some(e.0.1),
        }
            
    }
}

fn scan_dir(dir: PathBuf) -> Result<Directory, path::Error> {
    let entries = dir.read_dir().into_report().map_err(|e| {
        e.change_context(path::Error {
            offender: dir.display().to_string(),
            reason: "Failed to read directory".to_string(),
        })
    })?;

    let mut directory = Directory::new();

    for entry in entries {
        let entry = entry.into_report().map_err(|e| {
            e.change_context(path::Error {
                offender: dir.display().to_string(),
                reason: "Failed to read directory".to_string(),
            })
        })?;
        let path = entry.path();

        if path.is_file() {
            let path_string = path.display().to_string();
            let path = AbsPath::try_from(path).into_report().map_err(|e| {
                e.change_context(path::Error {
                    offender: path_string,
                    reason: "Failed to read entry in directory".to_string(),
                })
            })?;

            if let Some(p) = path.get_txtpp_file() {
                directory.input_files.push(p);
            }
        } else if path.is_dir() {
            directory.directories.push(path);
        }
    }

    Ok(directory)
}
