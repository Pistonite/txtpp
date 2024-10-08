use crate::error::PathError;
use crate::fs::{AbsPath, Directory, TxtppPath};
use error_stack::{Result, ResultExt};

pub fn scan_dir(dir: &AbsPath, recursive: bool) -> Result<Directory, PathError> {
    let dir_path = dir.as_path_buf();
    let entries = dir_path
        .read_dir()
        .change_context_lazy(|| PathError::from(&dir_path))
        .attach_printable("failed to read directory")?;

    let mut directory = Directory::new();

    for entry in entries {
        let entry = entry
            .change_context_lazy(|| PathError::from(&dir_path))
            .attach_printable("failed to read directory entry")?;
        let path = entry.path();

        if path.is_file() {
            if path.is_txtpp_file() {
                let path_abs = dir.share_base(path)?;
                directory.files.push(path_abs);
            }
        } else if path.is_dir() && recursive {
            let path_abs = dir.share_base(path)?;
            directory.subdirs.push(path_abs);
        }
    }

    Ok(directory)
}
