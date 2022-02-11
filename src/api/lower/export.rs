use crate::api::lower::error_codes::could_not_create_directory;
use std::{
    fs::create_dir_all,
    path::{Path, PathBuf}
};

/// Creates a export directory, appending an integer to create a unique directory if needed
pub fn get_export_destination_directory(destination_directory: &Path) -> PathBuf {
    let mut finalized_path = destination_directory.to_path_buf();
    if destination_directory.exists() {
        // if it exists, then append a number to the path and check if it too exits.
        // repeat until a non-existing path is found        
        let mut count = 1;
        loop {
            finalized_path.push(format!("msg-store-backup-{}", count));
            if !finalized_path.exists() {
                break;
            }
            finalized_path.pop();
            count += 1;
        }
    }
    finalized_path
}

pub fn create_export_directory(export_directory: &Path) -> Result<bool, &'static str> {
    if export_directory.exists() {
        if let Err(error) = create_dir_all(export_directory) {
            return Err(could_not_create_directory(file!(), line!(), error.to_string()))
        }
        return Ok(true)
    }
    Ok(false)
}