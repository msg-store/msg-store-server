use crate::{
    fmt_result
};
use dirs::{
    home_dir
};
use msg_store::{
    store::{
        GroupId,
        MsgByteSize
    }
};
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
        create_dir_all,
        read_to_string
    },
    path::{
        Path,
        PathBuf
    }
};

enum Ext {
    Json,
    Toml,
    Yaml,
    Yml
}
impl Ext {
    pub fn from_path(file_name: &Path) -> Ext {
        match file_name.extension() {
            Some(ext) => match ext.to_str() {
                Some(ext_str) => match ext_str {
                    "json" => Ext::Json,
                    "toml" => Ext::Toml,
                    "yaml" => Ext::Yaml,
                    "yml" => Ext::Yml,
                    _ => panic!("Invalid config file type.")
                },
                None => panic!("Invalid config file type.")
            },
            None => panic!("Invalid config file type.")
        }
    }
    // pub fn to_string(&self) -> String {
    //     match &self {
    //         Self::Json => "json".to_string(),
    //         Self::Toml => "toml".to_string(),
    //         Self::Yaml => "yaml".to_string(),
    //         Self::Yml => "yml".to_string()
    //     }
    // }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GroupConfig {
    pub priority: GroupId,
    pub max_byte_size: Option<MsgByteSize>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StoreConfig {
    pub location: Option<PathBuf>,
    pub max_byte_size: Option<MsgByteSize>,
    pub groups: Option<Vec<GroupConfig>>
}
impl StoreConfig {
    pub fn open(location: &Path) -> StoreConfig {
        let ext = Ext::from_path(location);
        let config: StoreConfig = {
            if location.exists() {
                let contents: String = read_to_string(location)
                    .expect("Could not read file.");
                match ext {
                    Ext::Json => from_json_str(&contents).expect("Invalid JSON config."),
                    _ => panic!("File type not yet supported.")
                }
            } else {
                let mut db_dir_location = home_dir().expect("Could not find home dir.");
                db_dir_location.push("msg-stored");
                create_dir_all(&db_dir_location).expect("Could not create db location dir.");
                StoreConfig {
                    location: Some(db_dir_location),
                    max_byte_size: None,
                    groups: None
                }
            }
        };
        config
    }
    pub fn update_config_file(&self, location: &Path) -> Result<(), String> {
        let contents = match Ext::from_path(location) {
            Ext::Json => fmt_result!(to_json_string(&self))?,
            _ => {
                return Err("File type not yet supported.".to_string())
            }
        };
        fmt_result!(fs::write(location, contents))?;
        Ok(())
    }
}

// check if location exists
// if not -> create location & config file
// validate config file
// if conf is invalid, panic
// read config
