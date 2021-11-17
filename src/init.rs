use std::{path::PathBuf, str::FromStr};

use clap::{Arg,App};
fn get_app<'a>() -> App<'a, 'a> {
    App::new("msg-store-server")
        .version("0.1.0")
        .author("Joshua Enokson <kilograhm@pm.me>")
        .about("A priority message store")
}

cfg_if::cfg_if! {
    if #[cfg(feature = "mem")] {

        use crate::config::StoreConfig;
        use msg_store::{ MemStore, open };
        pub type Store = MemStore;

        pub fn init() -> (Store, StoreConfig, Option<PathBuf>) {
            let matches = get_app()
                .arg(Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file"))
                .get_matches();

            let store_config = match matches.value_of("config") {
                Some(location) => StoreConfig::open(Some(PathBuf::from_str(location).expect("Invalid path"))),
                None => StoreConfig::open(None)
            };

            let config_location = match matches.value_of("config") {
                Some(location) => Some(PathBuf::from_str(location).expect("Invalid path")),
                None => None
            };

            (open(), store_config, config_location)
        }

    } else if #[cfg(feature = "level")] {

        use crate::config::StoreConfig;
        use msg_store_plugin_leveldb::{LevelStore, open};
        use std::fs::create_dir_all;
        use std::path::Path;

        pub type Store = LevelStore;
        pub fn init() -> (Store, StoreConfig, Option<PathBuf>) {
            let matches = get_app()
                .arg(Arg::with_name("config")
                    .short("c")
                    .long("config")
                    .value_name("FILE")
                    .help("Sets a custom config file"))
                .arg(Arg::with_name("level-dir")
                    .short("l")
                    .long("level-dir")
                    .value_name("DIR")
                    .help("Sets the database location")
                    .takes_value(true))

            let store_config = match matches.value_of("config") {
                Some(location) => StoreConfig::open(Some(PathBuf::from_str(location).expect("Invalid path"))),
                None => StoreConfig::open(None)
            };

            let config_location = match matches.value_of("config") {
                Some(location) => Some(PathBuf::from_str(location).expect("Invalid path")),
                None => None
            };

            let location = match matches.value_of("level-dir") {
                Some(location) => Path::new(location).to_path_buf(),
                None => match &store_config.level_db_location {
                    Some(location) => location.to_owned(),
                    None => panic!("location was not set")
                }
            };

            if !location.exists() {
                create_dir_all(location.clone()).expect("Could not create config dir");
            }            

            (open(location.as_path()), store_config, config_location)

        }

    } else {
        fn foo() { /* fallback implementation */ }
    }
}

