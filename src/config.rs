use crate::{
    fmt_result
};
// use dirs::{
//     home_dir
// };
// use msg_store::Store;
use serde::{
    Deserialize,
    Serialize
};
use serde_json::{
    to_string_pretty as to_json_string,
    from_str as from_json_str
};
use std::{
    fs::{
        self,
        read_to_string
    },
    path::{
        PathBuf
    }
};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GroupConfig {
    pub priority: i32,
    pub max_byte_size: Option<i32>
}

cfg_if::cfg_if! {

    if #[cfg(feature = "mem")] {

        #[derive(Debug, Deserialize, Serialize)]
        pub struct StoreConfig {
            pub host: Option<String>,
            pub port: Option<i32>,
            pub max_byte_size: Option<i32>,
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
            pub port: Option<i32>,
            pub max_byte_size: Option<i32>,
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
        // let config_path = home_dir().expect("Could not find home dir").join(".msg-store/config.json");
        let contents = fmt_result!(to_json_string(&self))?;
        fmt_result!(fs::write(config_path, contents))?;
        Ok(())
    }
}

// check if location exists
// if not -> create location & config file
// validate config file
// if conf is invalid, panic
// read config
