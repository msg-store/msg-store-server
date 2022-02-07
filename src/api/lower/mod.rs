pub use msg_store::{Store as MsgStore, Uuid};
use msg_store_db_plugin::Db;
use std::{
    collections::BTreeSet,
    sync::Mutex
};

pub mod msg;

pub mod stats {
    pub struct Stats {
        pub inserted: u32,
        pub deleted: u32,
        pub pruned: u32
    }
    impl Stats {
        pub fn new() -> Stats {
            Stats {
                inserted: 0,
                deleted: 0,
                pruned: 0
            }
        }
        pub fn add(&mut self, inserted: u32, deleted: u32, pruned: u32) {
            self.inserted += inserted;
            self.deleted += deleted;
            self.pruned += pruned;
        }
        pub fn replace(&mut self, inserted: u32, deleted: u32, pruned: u32) {
            self.inserted = inserted;
            self.deleted = deleted;
            self.pruned = pruned;
        }
    }
    
}

pub fn test(stats: Mutex<stats::Stats>) {
    let stats = stats.lock().unwrap();
    stats.add(1, 2, 3);
}

pub type Store = Mutex<MsgStore>;
pub type FileList = Mutex<BTreeSet<Uuid>>;
pub type Database<'a> = Mutex<Box<dyn Db<&'a [u8]>>>;