use clap::{Arg, App};
use std::{
    path::{
        PathBuf
    }, 
    str::FromStr
};

pub struct Args {
    pub config_location: PathBuf,
    pub database_location: Option<PathBuf>
}
impl Args {
    pub fn default() -> Args {
        let matches = App::new("msg-stored")
            .version("1.0")
            .author("Joshua Enokson <kilograhm@pm.me>")
            .about("Does awesome things")
            .arg(Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("CONFIG")
                .help("Sets a config file location")
                .required(true)
                .takes_value(true)
            )
            .arg(Arg::with_name("database")
                .short("d")
                .long("database")
                .value_name("DATABASE")
                .help("Sets the location of the database")
                // .required(true)
                .takes_value(true)
            )
            .get_matches();
    
        let config_location = match matches.value_of("config") {
            Some(location_str) => PathBuf::from_str(location_str).expect("Could get config location"),
            None => panic!("Could not get config location")
        };
        let database_location = match matches.value_of("database") {
            Some(location_str) => Some(PathBuf::from_str(location_str).expect("Could get config location")),
            None => None
        };
    
        Args {
            config_location,
            database_location
        }
    
    }
}
