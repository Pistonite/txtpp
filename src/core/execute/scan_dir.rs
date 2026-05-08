use cu::pre::*;

use crate::fs::{AbsPath, Directory, TxtppPath, normalize_path};

pub fn scan_dir(dir: &AbsPath, recursive: bool) -> cu::Result<Directory> {
    let dir_path = dir.as_path_buf();
    let entries = cu::check!(
        dir_path.read_dir(),
        "failed to read directory: '{}'",
        normalize_path(&dir_path.to_string_lossy())
    )?;

    let mut directory = Directory::new();

    for entry in entries {
        let entry = cu::check!(
            entry,
            "failed to read directory entry: '{}'",
            normalize_path(&dir_path.to_string_lossy())
        )?;
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
