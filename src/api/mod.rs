pub mod group;
pub mod group_defaults;
pub mod stats;
pub mod store;
pub mod msg;

use crate::config::StoreConfig;
use std::path::PathBuf;

pub fn update_config(config: &StoreConfig, config_path: &Option<PathBuf>) -> Result<(), String> {
    let should_update = {
        let mut should_update = true;
        if let Some(no_update) = config.no_update {
            if no_update {
                should_update = false;
            }
        }
        should_update
    };
    if should_update {
        if let Some(config_path) = config_path {
            config.update_config_file(&config_path)?;
        }        
    }
    Ok(())
}
