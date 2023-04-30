use crate::error::PathError;
use crate::fs::{AbsPath, Directory, TxtppPath};
use error_stack::{Report, Result};

pub fn resolve_inputs(inputs: &[String], base_abs_path: &AbsPath) -> Result<Directory, PathError> {
    let mut directory = Directory::new();
    for input in inputs {
        let input_path = base_abs_path.as_path().join(input);
        if input_path.is_dir() {
            let abs_path = base_abs_path.share_base(input_path)?;
            // if input is directory, add to the directories to scan
            directory.subdirs.push(abs_path);
        } else if !input_path.is_txtpp_file() {
            // input is a file but not a txtpp file
            // not that input file doesn't have to exist
            if let Some(input_path) = input_path.get_txtpp_file() {
                let abs_path = base_abs_path.share_base(input_path)?;
                directory.files.push(abs_path);
            } else {
                return Err(Report::new(PathError::from(&input_path)).attach_printable(
                    "file does not exist and corresponding txtpp file not found.",
                ));
            }
        } else {
            // input is txtpp file. it must exist
            let abs_path = base_abs_path.share_base(input_path)?;
            directory.files.push(abs_path);
        }
    }
    Ok(directory)
}
