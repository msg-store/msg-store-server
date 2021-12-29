use serde::{Deserialize, Serialize};
use serde_json::{from_str as from_json_str, to_string_pretty as to_json_string};
use std::{
    fs::{self, read_to_string},
    path::PathBuf,
};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GroupConfig {
    pub priority: u32,
    pub max_byte_size: Option<u32>,
}

cfg_if::cfg_if! {

    if #[cfg(feature = "mem")] {

        #[derive(Debug, Deserialize, Serialize)]
        pub struct StoreConfig {
            pub host: Option<String>,
            pub port: Option<u32>,
            pub max_byte_size: Option<u32>,
            pub groups: Option<Vec<GroupConfig>>,
            pub no_update: Option<bool>
        }
        impl StoreConfig {
            pub fn new() -> StoreConfig {
                StoreConfig {
                    host: Some("127.0.0.1".to_string()),
                    port: Some(8080),
                    max_byte_size: None,
                    groups: None,
                    no_update: Some(false)
                }
            }
        }

    } else {

        #[derive(Debug, Deserialize, Serialize, Clone)]
        pub struct LeveldbConfig {
            pub location: Option<PathBuf>
        }

        #[derive(Debug, Deserialize, Serialize)]
        pub struct StoreConfig {
            pub host: Option<String>,
            pub port: Option<u32>,
            pub max_byte_size: Option<u32>,
            pub groups: Option<Vec<GroupConfig>>,
            pub leveldb: Option<LeveldbConfig>,
            pub no_update: Option<bool>
        }
        impl StoreConfig {
            pub fn new() -> StoreConfig {
                StoreConfig {
                    host: Some("127.0.0.1".to_string()),
                    port: Some(8080),
                    max_byte_size: None,
                    groups: None,
                    leveldb: Some(LeveldbConfig {
                        location: None
                    }),
                    no_update: None
                }
            }
        }
    }

}

impl StoreConfig {
    pub fn open(config_path: PathBuf) -> StoreConfig {
        let contents: String = read_to_string(config_path).expect("Could not read config");
        from_json_str(&contents).expect("Invalid JSON config.")
    }
    pub fn update_config_file(&self, config_path: &PathBuf) -> Result<(), String> {
        let contents = match to_json_string(&self) {
            Ok(contents) => Ok(contents),
            Err(error) => Err(error.to_string()),
        }?;
        if let Err(error) = fs::write(config_path, contents) {
            return Err(error.to_string());
        };
        Ok(())
    }
}

// check if location exists
// if not -> create location & config file
// validate config file
// if conf is invalid, panic
// read config
