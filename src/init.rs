use clap::{App, Arg};
use dirs::home_dir;
use log::{error, debug};
use msg_store::{Store, Uuid};
use msg_store_db_plugin::{Db, Bytes};
use msg_store_plugin_leveldb::Leveldb;
// use serde_json::to_string_pretty;
use std::{
    collections::{BTreeMap},
    fs::create_dir_all,
    path::PathBuf,
    process::exit,
    sync::{Arc, Mutex}
};

use crate::{
    api::{
        lower::{
            file_storage::{
                FileStorage,
                read_file_storage_direcotory,
                discover_files,
                rm_from_file_storage
            },
            stats::Stats
        }
    },
    config::StoreConfig
};

pub struct InitResult {
    pub host: String,
    pub store: Mutex<Store>,
    pub configuration: Mutex<StoreConfig>,
    pub db: Mutex<Box<dyn Db>>,
    pub configuration_path: Option<PathBuf>,
    pub file_storage: Option<Mutex<FileStorage>>,
    pub stats: Mutex<Stats>
}

const HOST: &'static str = "host";
const PORT: &'static str = "port";
const CONFIG: &'static str = "config";
const CONFIG_PATH: &'static str = "config-path";
const UPDATE: &'static str = "update";
const NO_UPDATE: &'static str = "no-update";
const DATABASE: &'static str = "database";
const LEVELDB_PATH: &'static str = "leveldb-path";
const FILE_STORAGE: &'static str = "file-storage";
const FILE_STORAGE_PATH: &'static str = "file-storage-path";


fn get_app<'a>() -> App<'a, 'a> {
    App::new("msg-store-server")
        .version("0.1.0")
        .author("Joshua Enokson <kilograhm@pm.me>")
        .about("A priority message store")
        .arg(
            Arg::with_name(HOST)
                .short("h")
                .long(HOST)
                .help("Sets the host address"),
        )
        .arg(
            Arg::with_name(PORT)
                .short("p")
                .long(PORT)
                .help("Sets the port number"),
        )
        .arg(
            Arg::with_name(CONFIG)
                .short("c")
                .long(CONFIG)
                .help("Use a configuration a file. Will create one at $HOME/.msg-store/.config.json if one does not exist."),
        )
        .arg(
            Arg::with_name(CONFIG_PATH)
                // .short("c")
                .long(CONFIG_PATH)
                .takes_value(true)
                .help("Sets a path for the config file")
        )
        .arg(
            Arg::with_name(NO_UPDATE)
                .long(NO_UPDATE)
                .conflicts_with(UPDATE)
                .help("Will not write updated config to disk"),
        )
        .arg(
            Arg::with_name(UPDATE)
                .long(UPDATE)
                .conflicts_with(NO_UPDATE)
                .help("Will write updated config to disk"),
        )
        .arg(
            Arg::with_name(DATABASE)
                .short("d")
                .long(DATABASE)
                .help("Determines the database to use. (memory or leveldb)")
        )
        .arg(
            Arg::with_name(LEVELDB_PATH)
                .long(LEVELDB_PATH)
                .help("Sets the leveldb database path")
                .takes_value(true)
        )
        .arg(
            Arg::with_name(FILE_STORAGE)
                .short("f")
                .long(FILE_STORAGE)
                .help("Sets the location of the messages held in files"),
        )
        .arg(
            Arg::with_name(FILE_STORAGE_PATH)
                // .short("f")
                .long(FILE_STORAGE_PATH)
                .takes_value(true)
                .help("Sets the location of the messages held in files"),
        )
}

struct MemDb {
    msgs: BTreeMap<Arc<Uuid>, Bytes>,
    byte_size_data: BTreeMap<Arc<Uuid>, u32>
}
impl MemDb {
    pub fn new() -> MemDb {
        MemDb {
            msgs: BTreeMap::new(),
            byte_size_data: BTreeMap::new()
        }
    }
}
impl Db for MemDb {
    fn add(&mut self, uuid: Arc<Uuid>, msg: Bytes, msg_byte_size: u32) -> Result<(), String> {
        self.msgs.insert(uuid.clone(), msg);
        self.byte_size_data.insert(uuid, msg_byte_size);
        Ok(())
    }
    fn get(&mut self, uuid: Arc<Uuid>) -> Result<Bytes, String> {
        match self.msgs.get(&uuid) {
            Some(msg) => Ok(msg.clone()),
            None => Err("msg not found".to_string())
        }
    }
    fn del(&mut self, uuid: Arc<Uuid>) -> Result<(), String> {
        self.msgs.remove(&uuid);
        self.byte_size_data.remove(&uuid);
        Ok(())
    }
    fn fetch(&mut self) -> Result<Vec<(Arc<Uuid>, u32)>, String> {
        Ok(vec![])
        // Ok(self.byte_size_data.into_iter().collect::<Vec<(Uuid, u32)>>())
    }
}

pub fn init() -> InitResult {

    let matches = get_app().get_matches();

    // configuration
    // get configuration path
    let mut configuration = StoreConfig::new();
    let configuration_path = {
        // get saved configuration path
        let configuration_path = {
            if let Some(configuration_path) = matches.value_of(CONFIG_PATH) {
                Some(PathBuf::from(configuration_path))
            } else {
                if matches.is_present(CONFIG) {
                    if let Some(home_dir) = home_dir() {
                        let mut configuration_path = home_dir;
                        configuration_path.push(".msg-store");
                        if let Err(error) = create_dir_all(&configuration_path) {
                            error!("ERROR_CODE: 45698eb2-9301-43ae-b9c0-f8d43f45ff18. Could not create .msg-store directory: {}", error.to_string());
                            exit(1);
                        }
                        configuration_path.push("config.json");                        
                        // create default configuration file
                        let defualt_configuration = StoreConfig::new();
                        if let Err(error) = defualt_configuration.update_config_file(&configuration_path) {
                            error!("ERROR_CODE: 4ea5b303-622d-438f-af17-829a9470688e. Could not write to configuration file: {}", error.to_string());
                            exit(1);
                        };
                        Some(configuration_path)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        };
        configuration_path
    };

    // get update config from saved config
    if let Some(configuration_path) = configuration_path.as_ref() {
        let saved_configuration = StoreConfig::open(configuration_path);
        // validate values from configuration file
        if let Some(database) = &saved_configuration.database {
            let database = database.to_ascii_lowercase();
            if database != "mem" && database != "memory" && database != "leveldb" {
                error!("ERROR_CODE: 8db53e5b-be6e-4f68-b2d0-c800c73bac98. Invalid database option in the configuration file. Expected mem, memory or leveldb");
                exit(1);
            }
        }
        if let Some(no_update) = configuration.no_update {
            if let Some(update) = configuration.update {
                if update == no_update {
                    error!("ERROR_CODE: 9805cb4a-26c4-44e0-be51-a4387902473d. The option '--no-update' cannot be used with '--update'");
                    exit(1);
                }
            }
        }
        configuration.inherit(saved_configuration);
    }

    // update config from cli options
    // get host, port, no-update, database, leveldb-path, file-storage, file-storage-path
    // update host from cli
    let host = {
        if let Some(host) = matches.value_of(HOST) {
            let host = host.to_string();
            configuration.host = Some(host.to_string());
            host
        } else {
            if let Some(host) = &configuration.host {
                host.to_string()
            } else {
                "127.0.0.1".to_string()
            }
        }
    };    
    // update port from cli
    let port = {
        if let Some(port) = matches.value_of(PORT) {
            let port = match port.parse::<u32>() {
                Ok(port) => port,
                Err(_error) => {
                    error!("ERROR_CODE: 562d0f17-332f-4c67-bc84-006ddd23e370. Invalid port option. Expected u32");
                    exit(1);
                }
            };
            configuration.port = Some(port);
            port
        } else {
            if let Some(port) = configuration.port {
                port
            } else {
                8080
            }
        }
    };
    // update no-update from cli
    if matches.is_present(NO_UPDATE) {
        configuration.no_update = Some(true);
    }
    // update update from cli
    if matches.is_present(UPDATE) {
        configuration.no_update = Some(true);
    }
    // validate update/no-update options
    if let Some(no_update) = configuration.no_update {
        if let Some(update) = configuration.update {
            if update == no_update {
                error!("ERROR_CODE: 2f72c8e6-8027-434f-837e-f6b0392baa95. The option '--no-update' cannot be used with '--update'");
                exit(1);
            }
        }
    }
    // update database, leveldb-path from cli
    if let Some(database) = matches.value_of(DATABASE) {
        // validate database option
        let database_lower = database.to_ascii_lowercase();
        if database_lower != "leveldb" {
            // validate leveldb path option
            let leveldb_path = {
                if let Some(database_path) = matches.value_of(LEVELDB_PATH) {
                    PathBuf::from(database_path)
                } else {
                    if let Some(level_db_path) = configuration.leveldb_path {
                        PathBuf::from(level_db_path)
                    } else {
                        if let Some(home_dir) = home_dir() {
                            let mut level_db_path = home_dir;
                            level_db_path.push(".msg-store/leveldb");
                            level_db_path
                        } else {
                            error!("ERROR_CODE: b2ceac74-524f-475b-ac79-f58f6614359e. Could not create leveldb path.");
                            exit(1);
                        }
                    }
                }
            };
            if let Err(error) = create_dir_all(&leveldb_path) {
                error!("ERROR_CODE: b9da6a07-be9e-40e2-9283-204fbd449edc. Could not create leveldb path: {}", error.to_string());
                exit(1);
            }
            configuration.leveldb_path = Some(leveldb_path);

        } else if database_lower != "mem" && database_lower != "memory" {
            error!("ERROR_CODE: 399cb52d-f50c-4508-b48c-af41cf77f007. Invalid database option. Expected mem, memory, or leveldb");
            exit(1);
        }
    }
    // update file-storage, file-storage-path from cli
    if let Some(file_storage_path) = matches.value_of(FILE_STORAGE_PATH) {
        debug!("file-storage-path option: {}", file_storage_path);
        configuration.file_storage_path = Some(PathBuf::from(file_storage_path));
    } else {
        debug!("file-storage-path option: none");
        if matches.is_present(FILE_STORAGE) {
            if let Some(home_dir) = home_dir() {
                let mut file_storage_path = home_dir;
                file_storage_path.push(".msg-store/file-storage");
                if let Err(error) = create_dir_all(&file_storage_path) {
                    error!("ERROR_CODE: 48c68d9d-cc29-4938-bf61-e5cb9b74ab70. Could not create default file storage path: {}", error.to_string());
                    exit(1);
                }
                configuration.file_storage_path = Some(file_storage_path);
            } else {
                error!("ERROR_CODE: f4a3bb5c-f2d1-43db-a9d0-ba1c3e1add46. Could not create default file storage path due to a home directory not being present.");
                exit(1);
            }
        }
    }
    // get database
    let mut database: Box<dyn Db> = {
        if let Some(database_type) = &configuration.database {
            let database_type = database_type.to_ascii_lowercase();
            if database_type == "mem" || database_type == "memory" {
                Box::new(MemDb::new())
            } else if database_type == "leveldb" {
                let level_db_path = match &configuration.leveldb_path {
                    Some(level_db_path) => level_db_path,
                    None => {
                        error!("ERROR_CODE: a4660649-87e5-4060-9ce7-365aeeda0231. Missing leveldb path");
                        exit(1);
                    }
                };
                let leveldb = match Leveldb::new(level_db_path) {
                    Ok(leveldb) => leveldb,
                    Err(error) => {
                        error!("ERROR_CODE: 6714d014-e336-4703-a8ff-e9593fd52ae8. {}", error);
                        exit(1);
                    } 
                };
                Box::new(leveldb)
            } else {
                error!("ERROR_CODE: ab444716-9ebc-4f46-a461-856d0d9952a5. The database option is invalid.");
                exit(1);
            }
        } else {
            error!("ERROR_CODE: a9a81e3f-f1ec-458f-99a4-0e857f6b714a. The database option is invalid.");
            exit(1);
        }
    };
    // get the stored messages from the database
    let msgs = match database.fetch() {
        Ok(msgs) => msgs,
        Err(error) => {
            error!("ERROR_CODE: 30a40861-2205-4d4c-a692-55da40b44b15. {}", error);
            exit(1);
        }
    };
    // get file list of all files stored on disk
    let mut file_storage: Option<FileStorage> = {
        if let Some(file_storage_path) = &configuration.file_storage_path {
            let mut file_storage = FileStorage::new(file_storage_path);
            let uuids = match read_file_storage_direcotory(file_storage_path) {
                Ok(index) => index,
                Err(error_code) => {
                    error!("ERROR_CODE: {}.", error_code);
                    exit(1);
                }
            };
            discover_files(&mut file_storage, uuids);
            Some(file_storage)
        } else {
            None
        }
    };

    
    // TODO: add config to store
    let mut store = Store::new();
    // add messages to store, prune excess
    let (removed_uuids, pruned_count) = {
        let mut removed_uuids = vec![];
        let mut pruned_count = 0;
        for (uuid, msg_byte_size) in &msgs {
            let mut add_result = match store.add_with_uuid(uuid.clone(), *msg_byte_size) {
                Ok(add_result) => add_result,
                Err(error) => {
                    // TODO: add error handling options to the configuration
                    error!("ERROR_CODE: 0ada6889-83e0-4bcf-98aa-613dffa11a3b. {}", error.to_string());
                    exit(1);
                }
            };
            for uuid_removed in &add_result.msgs_removed {
                if let Err(error) = database.del(uuid_removed.clone()) {
                    error!("ERROR_CODE: 4fbc000f-4ce4-4110-ab59-15df32fd9b4d. {}", error);
                    exit(1);
                }
            }
            removed_uuids.append(&mut add_result.msgs_removed);
            pruned_count += add_result.msgs_removed.len();
        }
        (removed_uuids, pruned_count as u32)
    };
    // removed pruned files if any
    if let Some(file_storage) = file_storage.as_mut() {
        for uuid in removed_uuids {
            if let Err(error_code) = rm_from_file_storage(file_storage, &uuid) {
                error!("ERROR_CODE: {}.", error_code);
                exit(1);
            }
        }
    }
    let stats = Stats { inserted: 0, deleted: 0, pruned: pruned_count };
    
    InitResult {
        host: format!("{}:{}", host, port),
        store: Mutex::new(store),
        db: Mutex::new(database),
        file_storage: match file_storage {
            Some(file_storage) => Some(Mutex::new(file_storage)),
            None => None
        },
        configuration: Mutex::new(configuration),
        configuration_path,
        stats: Mutex::new(stats)
    }

}
