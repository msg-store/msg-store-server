use clap::{App};

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

        pub fn init() -> (Store, StoreConfig) {
            let _matches = get_app()
                // .arg(Arg::with_name("config")
                // .short("c")
                // .long("config")
                // .value_name("FILE")
                // .help("Sets a custom config file"))
                .get_matches();

            let store_config = StoreConfig::open();

            (open(), store_config)
        }

    } else if #[cfg(feature = "level")] {

        use crate::config::StoreConfig;
        use msg_store_plugin_leveldb::{LevelStore, open};
        use std::fs::create_dir_all;
        use std::path::Path;
        use dirs::home_dir;

        pub type Store = LevelStore;
        pub fn init() -> (Store, StoreConfig) {
            let _matches = get_app()
                // .arg(Arg::with_name("config")
                //     .short("c")
                //     .long("config")
                //     .value_name("FILE")
                //     .help("Sets a custom config file"))
                // .arg(Arg::with_name("level-dir")
                //     .short("l")
                //     .long("level-dir")
                //     .value_name("DIR")
                //     .help("Sets the database location")
                //     .takes_value(true))
                .get_matches();

            let store_config = StoreConfig::open();

            let db_dir = home_dir().expect("Could not get home dir.").join(".msg-store/leveldb");

            if !db_dir.exists() {
                create_dir_all(db_dir.clone()).expect("Could not create leveldb dir");
            }            

            (open(db_dir.as_path()), store_config)

        }

    } else {
        fn foo() { /* fallback implementation */ }
    }
}

