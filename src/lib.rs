use actix_web::web::Payload;
// use crate::api::{lock_or_exit};
// use crate::{
//     config::StoreConfig
// };
use futures::StreamExt;
use msg_store::Uuid;
use log::{error,debug};
use std::{
    collections::BTreeSet,
    fs::{
        File
    },
    io::Write,
    path::PathBuf,
    process::exit,
    sync::{Mutex, MutexGuard},
};

use serde::{Serialize};

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Stats {
    pub inserted: u32,
    pub deleted: u32,
    pub pruned: u32
}
