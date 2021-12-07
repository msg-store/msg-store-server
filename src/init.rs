

use clap::{Arg, ArgMatches,App};
use dirs::home_dir;
use serde_json::{
    to_string_pretty
};
use std::{
    fs::{
        self, 
        create_dir
    },
    path::PathBuf
};

pub struct InitResult {
    pub host: String,
    pub store: Store,
    pub store_config: StoreConfig,
    pub config_location: Option<PathBuf>,
    pub update_config: bool
}

fn get_app<'a>() -> App<'a, 'a> {
    App::new("msg-store-server")
        .version("0.1.0")
        .author("Joshua Enokson <kilograhm@pm.me>")
        .about("A priority message store")
        .arg(Arg::with_name("host")
            .short("h")
            .long("host")
            .value_name("HOST")
            .help("Sets the host address"))
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .value_name("PORT")
            .help("Sets the port number"))
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("CONFIG")
            .help("Sets a custom config file"))
        .arg(Arg::with_name("no-update")
            // .short("c")
            .long("no-update")
            // .value_name("NOUPDATE")
            .help("Will not write updated config to disk"))
        .arg(Arg::with_name("no-config")
            .long("no-config")
            .conflicts_with("config")
            .conflicts_with("no-update")
            // .value_name("NOCONFIG")
            .help("Will not search for or load a config file"))
}

fn get_host(matches: &ArgMatches, store_config: &StoreConfig) -> String {
    let mut host = {
        if let Some(host) = matches.value_of("host") {
            host.to_string()
        } else {
            if let Some(host) = store_config.host.clone() {
                host
            } else {
                String::from("localhost")
            }                        
        }
    };
    let port = {
        if let Some(port) = matches.value_of("port") {
            port.to_string()
        } else {
            if let Some(port) = store_config.port {
                port.to_string()
            } else {
                String::from("8080")
            }                        
        }
    };
    host.push_str(":");
    host.push_str(port.as_str());
    host
}

fn get_update_config_setting(matches: &ArgMatches, store_config: &StoreConfig) -> bool {
    if let Some(noupdate) = store_config.no_update {
        noupdate
    } else {
        matches.is_present("no-update")
    }
}

fn get_store_config(config_location: &Option<PathBuf>) -> StoreConfig {
    if let Some(config_location) = config_location.clone() {
        StoreConfig::open(config_location)
    } else {
        StoreConfig::new()
    }
}

fn get_config_path(matches: &ArgMatches) -> Option<PathBuf> {
    if matches.is_present("no-config") {
        None
    } else {
        if let Some(config_location) = matches.value_of("config") {
            Some(PathBuf::from(config_location))
        } else {
            let msg_store_dir = PathBuf::new().join(home_dir().expect("Could not get home directory")).join(".msg-store");
            if !msg_store_dir.exists() {
                create_dir(msg_store_dir.clone()).expect("Could not create .msg-store dir");
            }                        
            let config_path = msg_store_dir.join("config.json");
            if !config_path.exists() {
                let contents = StoreConfig::new();
                fs::write(config_path.clone(), to_string_pretty(&contents).expect("Could not create config.json")).expect("Could not write to config.json");
            }
            Some(config_path)
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "mem")] {

        use crate::config::StoreConfig;
        use msg_store::{ MemStore, open };
        pub type Store = MemStore;

        pub fn init() -> InitResult {

            let matches = get_app().get_matches();
            let config_location = get_config_path(&matches);
            let store_config = get_store_config(&config_location);
            let update_config = get_update_config_setting(&matches, &store_config);

            InitResult {
                host: get_host(&matches, &store_config),
                store: open(),
                store_config,
                config_location,
                update_config
            }

        }

    } else if #[cfg(feature = "level")] {

        use crate::config::StoreConfig;
        use msg_store_plugin_leveldb::{LevelStore, open};
        use std::fs::create_dir_all;
        // use std::path::Path;
        // use dirs::home_dir;

        pub type Store = LevelStore;
        pub fn init() -> InitResult {
            
            let matches = get_app()
                .arg(Arg::with_name("leveldb-location")
                    .long("leveldb-location")
                    .help("Sets the database location")
                    .takes_value(true))
                .get_matches();

            let config_location = get_config_path(&matches);
            let store_config = get_store_config(&config_location);
            let update_config = get_update_config_setting(&matches, &store_config);

            // let store_config = StoreConfig::open();
            let leveldb_location = {
                if let Some(location) = matches.value_of("leveldb-location") {
                    PathBuf::from(location)
                } else {
                    if let Some(leveldb) = store_config.leveldb.clone() {
                        if let Some(location) = leveldb.location.clone() {
                            PathBuf::from(location)
                        } else {
                            panic!("Leveldb location is not set.");
                        }
                    } else {
                        panic!("Leveldb location is not set.");
                    }
                }
            };

            // let db_dir = home_dir().expect("Could not get home dir.").join(".msg-store/leveldb");

            if !leveldb_location.exists() {
                create_dir_all(leveldb_location.clone()).expect("Could not create leveldb location");
            }
            
            InitResult {
                host: get_host(&matches, &store_config),
                store: open(leveldb_location.as_path()).unwrap(),
                store_config,
                config_location,
                update_config
            }

        }

    } else {
        fn foo() { /* fallback implementation */ }
    }
}

