use log::error;
use serde::{Deserialize, Serialize};
use serde_json::{from_str as from_json_str, to_string_pretty as to_json_string};
use std::{
    fs::{self, read_to_string},
    path::{Path, PathBuf},
    process::exit
};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GroupConfig {
    pub priority: u32,
    pub max_byte_size: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StoreConfig {
    pub host: Option<String>,
    pub port: Option<u32>,
    pub database: Option<String>,
    pub leveldb_path: Option<PathBuf>,
    pub file_storage: Option<bool>,
    pub file_storage_path: Option<PathBuf>,
    pub max_byte_size: Option<u32>,
    pub groups: Option<Vec<GroupConfig>>,
    pub no_update: Option<bool>,
    pub update: Option<bool>
}

impl StoreConfig {
    pub fn new() -> StoreConfig {
        StoreConfig {
            host: Some("127.0.0.1".to_string()),
            port: Some(8080),
            database: Some("mem".to_string()),
            leveldb_path: None,
            file_storage: Some(false),
            file_storage_path: None,
            max_byte_size: None,
            groups: None,
            no_update: None,
            update: Some(true)
        }
    }
    pub fn open(config_path: &Path) -> StoreConfig {
        let contents: String = match read_to_string(config_path) {
            Ok(content) => content,
            Err(error) => {
                error!("ERROR_CODE: 9cf571a9-fe29-4737-a977-d5ad580e1b28. Could not read configuration: {}", error.to_string());
                exit(1);
            }
        };
        match from_json_str(&contents) {
            Ok(config) => config,
            Err(error) => {
                error!("ERROR_CODE: 95d9ece7-39bb-41fb-93a7-a530786673d3. Could not parse configuration: {}", error.to_string());
                exit(1);
            }
        }
    }
    pub fn update_config_file(&self, config_path: &Path) -> Result<(), String> {
        let contents = match to_json_string(&self) {
            Ok(contents) => Ok(contents),
            Err(error) => Err(error.to_string()),
        }?;
        if let Err(error) = fs::write(config_path, contents) {
            return Err(error.to_string());
        };
        Ok(())
    }
    pub fn to_json(&self) -> Result<String, String> {
        match to_json_string(&self) {
            Ok(json) => Ok(json),
            Err(error) => Err(error.to_string())
        }
    }
    pub fn inherit(&mut self, configuration: Self) {
        self.host = configuration.host;
        self.port = configuration.port;
        self.database = configuration.database;
        self.leveldb_path = configuration.leveldb_path;
        self.file_storage = configuration.file_storage;
        self.file_storage_path = configuration.file_storage_path;
        self.max_byte_size = configuration.max_byte_size;
        self.groups = configuration.groups;
        self.no_update = configuration.no_update;
    }
}
