use crate::{
    fmt_result
};
use dirs::{
    home_dir
};
use serde::{
    Deserialize,
    Serialize
};
use serde_json::{
    to_string_pretty as to_json_string,
    from_str as from_json_str,
    json
};
use std::{
    fs::{
        self,
        create_dir_all,
        read_to_string
    },
    path::{
        Path,
        PathBuf
    },
    str::FromStr
};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GroupConfig {
    pub priority: i32,
    pub max_byte_size: Option<i32>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StoreConfig {
    pub level_db_location: Option<PathBuf>,
    pub max_byte_size: Option<i32>,
    pub groups: Option<Vec<GroupConfig>>
}


impl StoreConfig {
    pub fn new() -> StoreConfig {
        StoreConfig {
            level_db_location: None,
            max_byte_size: None,
            groups: None
        }
    }
    pub fn search() -> Option<PathBuf> {
        let create_config_file = |location: &PathBuf| {
            let config_path = location.join("config.json");
            create_dir_all(location.clone()).expect("Could not create config dir");
            cfg_if::cfg_if! {
                if #[cfg(feature = "mem")] {
                    let contents = json!({
                        "max_byte_size": null,
                        "groups": []
                    }).to_string();
                } else {        
                    let level_location = location.join("leveldb");            
                    let contents = json!({
                        "level_db_location": level_location.to_str(),
                        "max_byte_size": null,
                        "groups": []
                    }).to_string();
                }
            }
            fs::write(config_path, contents).expect("Could not write to ~/.local/share/msg-store/config.json");
        };
        
        // search in ~/.local/**/msg-store; ~/.msg-store; /etc/msg-store/
        // check if ~/ exists
        if let Some(home) = home_dir() {
            // check if ~/.local exists
            let local_path = home.clone().join(".local");
            let local_share_path = local_path.join("share");
            let local_share_msg_store_dir_path = local_share_path.join("msg-store");
            let local_config_path = local_share_msg_store_dir_path.join("config.json");
            let home_msg_store_path = home.join(".msg-store");
            let home_config_path = home_msg_store_path.join("config.json");
            if local_path.exists() && local_share_path.exists() {
                // check if ~/.local/share/msg-store exists
                if local_share_msg_store_dir_path.exists() {
                    // check if ~/.local/share/msg-store/config.json exists
                    if local_config_path.exists() {
                        return Some(local_config_path);
                    } else {
                        // check if ~/.msg-store exists
                        if home_config_path.exists() {
                            return Some(home_config_path);
                        } else {
                            // check if /etc/msg-store/config.json exists
                            let etc_config_path = PathBuf::from_str("/etc/msg-store/config.json").expect("Could not create path from str");
                            if etc_config_path.exists() {
                                return Some(etc_config_path);
                            } else {
                                // create ~/.local/share/msg-store/config.json
                                create_config_file(&local_share_msg_store_dir_path);
                                return Some(local_config_path);
                            }
                        }
                    }
                } else {
                    // check if ~/.msg-store exists
                    if home_config_path.exists() {
                        return Some(home_config_path);
                    } else {
                        // check if /etc/msg-store/config.json exists
                        let etc_config_path = PathBuf::from_str("/etc/msg-store/config.json").expect("Could not create path from str");
                        if etc_config_path.exists() {
                            return Some(etc_config_path);
                        } else {                            
                            // create ~/.msg-store/config.json
                            create_config_file(&home_msg_store_path);
                            return Some(home_config_path);
                        }
                    }
                }
            } else {
                // check if ~/.msg-store exists
                let home_msg_store_path = home.join(".msg-store");
                let home_config_path = home_msg_store_path.join("config.json");
                if home_config_path.exists() {
                    return Some(home_config_path);
                } else {
                    // check if /etc/msg-store/config.json exists
                    let etc_config_path = PathBuf::from_str("/etc/msg-store/config.json").expect("Could not create path from str");
                    if etc_config_path.exists() {
                        return Some(etc_config_path);
                    } else {
                        // create ~/.msg-store/config.json
                        create_config_file(&home_msg_store_path);
                        return Some(home_config_path);
                    }
                }
            }
        } else {
            let etc_config = PathBuf::from_str("/etc/msg-store/config.json").expect("Could create path from str");
            if etc_config.exists() {
                let contents: String = read_to_string(etc_config).expect("Could not read file.");
                return Some(from_json_str(&contents).expect("Invalid JSON config."))
            } else {
                return None
            }
        }
    }
    pub fn open(location: Option<PathBuf>) -> StoreConfig {
        if let Some(location) = location {
            let contents: String = read_to_string(location).expect("Could not read config");
            from_json_str(&contents).expect("Invalid JSON config.")
        } else {
            if let Some(location) = Self::search() {
                let contents: String = read_to_string(location).expect("Could not read config");
                from_json_str(&contents).expect("Invalid JSON config.")
            } else {
                StoreConfig::new()
            }
        }

    }
    pub fn update_config_file(&self, location: &Path) -> Result<(), String> {
        let contents = fmt_result!(to_json_string(&self))?;
        fmt_result!(fs::write(location, contents))?;
        Ok(())
    }
}

// check if location exists
// if not -> create location & config file
// validate config file
// if conf is invalid, panic
// read config
