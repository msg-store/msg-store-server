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
        read_to_string
    }
};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GroupConfig {
    pub priority: i32,
    pub max_byte_size: Option<i32>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StoreConfig {
    pub max_byte_size: Option<i32>,
    pub groups: Option<Vec<GroupConfig>>
}


impl StoreConfig {
    pub fn new() -> StoreConfig {
        StoreConfig {
            max_byte_size: None,
            groups: None
        }
    }
    
    pub fn open() -> StoreConfig {
        if let Some(home_dir) = home_dir() {
            let msg_store_dir = home_dir.join(".msg-store");
            if !msg_store_dir.exists() {
                fs::create_dir(msg_store_dir.clone()).expect("Could not create msg store dir");
            }
            let config_path = msg_store_dir.join("config.json");
            if !config_path.exists() {
                cfg_if::cfg_if! {
                    if #[cfg(feature = "mem")] {
                        let contents = json!({
                            "max_byte_size": null,
                            "groups": []
                        }).to_string();
                    } else {        
                        let contents = json!({
                            "max_byte_size": null,
                            "groups": []
                        }).to_string();
                    }
                }
                fs::write(config_path.clone(), contents).expect("Could not write to ~/.local/share/msg-store/config.json");
            }
            let contents: String = read_to_string(config_path).expect("Could not read config");
            from_json_str(&contents).expect("Invalid JSON config.")
        } else {
            panic!("Could not find home directory");
        }
    }
    pub fn update_config_file(&self) -> Result<(), String> {
        let config_path = home_dir().expect("Could not find home dir").join(".msg-store/config.json");
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
