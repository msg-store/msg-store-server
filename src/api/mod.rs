pub mod export;
pub mod group;
pub mod group_defaults;
pub mod msg;
pub mod stats;
pub mod store;
pub mod ws;
pub mod lower;

use crate::config::StoreConfig;
use actix_web::HttpResponse;
use actix_web_actors::ws::WebsocketContext;
use log::{info, error};
use msg_store::Uuid;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_value, json, to_string, Value};
use std::{
    collections::BTreeSet,
    fmt::Display, 
    fs::{remove_file, read_dir},
    path::{Path, PathBuf}, 
    process::exit,
    sync::{Arc, Mutex, MutexGuard}
};

use self::ws::Websocket;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Stats {
    pub inserted: u32,
    pub deleted: u32,
    pub pruned: u32
}

pub struct FileManager {
    file_storage_path: PathBuf,
    file_list: BTreeSet<Arc<Uuid>>
}
impl FileManager {
    pub fn new(file_storage_path: &Path) -> Self {
        FileManager { file_storage_path: file_storage_path.to_path_buf(), file_list: BTreeSet::new() }
    }
    pub fn open(file_storage_path: &Path) -> Self {
        let mut file_manager = Self::new(file_storage_path);
        let dir_entries = match read_dir(&file_manager.file_storage_path) {
            Ok(entries) => entries,
            Err(error) => {
                error!("ERROR_CODE: a3eed70c-699a-4fb8-b171-eda1de93b00e. Could not read file storage directory: {}", error.to_string());
                exit(1);
            }
        };
        for entry in dir_entries.into_iter() {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    error!("ERROR_CODE: be5de83e-a766-415d-ae46-ae378a2cd3ba. Could not read entry: {}", error.to_string());
                    exit(1);
                }
            };
            let file_name = match entry.file_name().into_string() {
                Ok(file_name) => file_name,
                Err(error) => {
                    error!("ERROR_CODE: 437335a6-e019-41fb-a5bc-3825aa06a0cb. Could not parse file name {:#?}: {:#?}", entry.file_name(), error);
                    exit(1);
                }
            };
            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(error) => {
                    error!("ERROR_CODE: 83e6e00c-92f5-4855-870a-86e76eef2d4a. Could not get entry metadata from {}: {}", file_name, error.to_string());
                    exit(1);
                }
            };
            if !metadata.is_file() {
                error!("ERROR_CODE: 8fe85904-70de-4429-aa46-090ca7acd893. Non-file found in file storage directory: {}", file_name);
                exit(1);
            }                
            let uuid = match Uuid::from_string(&file_name) {
                Ok(uuid) => uuid,
                Err(error) => {
                    error!("ERROR_CODE: b7065e7d-60d4-4ab9-9083-14ce55123e03. Could not parse uuid from file name {}: {}", file_name, error);
                    exit(1);
                }
            };
            file_manager.file_list.insert(uuid);
        }
        file_manager
    }
    pub fn del_batch(&mut self, uuids: Vec<Arc<Uuid>>)  {
        for uuid in uuids {
            if !self.file_list.remove(&uuid) {
                return;
            }
            let uuid_string = uuid.to_string();
            let mut file_path = self.file_storage_path.to_path_buf();
            file_path.push(uuid_string);
            if let Err(error) = remove_file(file_path) {
                error!("ERROR_CODE: 675e40c1-14d7-40c3-9491-29e0c25436a1. Could not remove file: {}", error.to_string());
                exit(1);
            }
        }
    }
}

pub fn update_config(config: &StoreConfig, config_path: &Option<PathBuf>) -> Result<(), String> {
    let should_update = {
        let mut should_update = true;
        if let Some(no_update) = config.no_update {
            if no_update {
                should_update = false;
            }
        }
        should_update
    };
    if should_update {
        if let Some(config_path) = config_path {
            config.update_config_file(&config_path)?;
        }
    }
    Ok(())
}

pub enum Reply<T: Serialize> {
    Ok,
    OkWData(T),
    BadRequest(String),
    Conflict(String),
}

pub fn from_value_prop<'a, T: DeserializeOwned, T2: Into<String> + Display>(
    value: &Value,
    prop: &'static str,
    prop_type: T2,
) -> Result<Option<T>, String> {
    if let Some(value) = value.get(prop) {
        if let Ok(value) = from_value::<Option<T>>(value.clone()) {
            Ok(value)
        } else {
            Err(format!("/data/{} must be type {}", prop, prop_type))
        }
    } else {
        Ok(None)
    }
}

pub fn from_value_prop_required<'a, T: DeserializeOwned>(
    value: &Value,
    prop: &'static str,
    prop_type: &'static str,
) -> Result<T, String> {
    if let Some(value) = value.get(prop) {
        if let Ok(value) = from_value::<Option<T>>(value.clone()) {
            if let Some(value) = value {
                Ok(value)
            } else {
                Err(format!("/data/{} must be type {}", prop, prop_type))
            }
        } else {
            Err(format!("/data/{} must be type {}", prop, prop_type))
        }
    } else {
        Err(format!("/data/{} must be type {}", prop, prop_type))
    }
}

pub fn get_optional_number(value: &Value, prop: &'static str) -> Result<Option<u32>, String> {
    from_value_prop::<u32, _>(value, prop, "number")
}

pub fn get_optional_string(value: &Value, prop: &'static str) -> Result<Option<String>, String> {
    from_value_prop::<String, _>(value, prop, "string")
}

pub fn get_required_string(value: &Value, prop: &'static str) -> Result<String, String> {
    from_value_prop_required(value, prop, "string")
}

pub fn get_required_priority(value: &Value) -> Result<u32, String> {
    from_value_prop_required(value, "priority", "number")
}

pub fn get_optional_priority(value: &Value) -> Result<Option<u32>, String> {
    from_value_prop::<u32, _>(value, "priority", "number")
}

pub fn get_optional_max_byte_size(value: &Value) -> Result<Option<u32>, String> {
    let mut max_byte_size = from_value_prop(value, "max_byte_size", "number")?;
    if let None = max_byte_size {
        max_byte_size = from_value_prop(value, "maxByteSize", "number")?;
    }
    Ok(max_byte_size)
}

pub fn validate_uuid_string(uuid_string: String) -> Result<Arc<Uuid>, String> {
    match Uuid::from_string(&uuid_string) {
        Ok(uuid) => Ok(uuid),
        Err(_) => Err(format!("uuid string must be of <u128>-<u32>")),
    }
}

// pub fn get_optional_uuid(value: &Value) -> Result<Option<Uuid>, String> {
//     match from_value_prop::<String, _>(value, "uuid", "string") {
//         Ok(uuid_string) => match uuid_string {
//             Some(uuid_string) => match validate_uuid_string(uuid_string) {
//                 Ok(uuid) => Ok(Some(uuid)),
//                 Err(message) => Err(message),
//             },
//             None => Ok(None),
//         },
//         Err(message) => Err(message),
//     }
// }

pub fn get_required_uuid(value: &Value) -> Result<Arc<Uuid>, String> {
    match from_value_prop_required::<String>(value, "uuid", "string") {
        Ok(uuid_string) => match validate_uuid_string(uuid_string) {
            Ok(uuid) => Ok(uuid),
            Err(message) => Err(message),
        },
        Err(message) => Err(message),
    }
}

// pub fn get_require_msg(value: &Value) -> Result<String, String> {
//     from_value_prop_required::<String>(value, "msg", "string")
// }

pub fn ws_ok(cmd: &str) -> String {
    format_log_complete::<()>(&format!("/api/ws {}", cmd), 200, None);
    match to_string(&json!({ "cmd": cmd,  "status": 200 })) {
        Ok(msg) => msg,
        Err(error) => {
            error!("ERROR_CODE: 9f7c13bc-483d-4076-bc6d-adba42e1958f. Could not convert response to string: {}", error.to_string());
            exit(1);
        },
    }
}

pub fn ws_ok_w_data<T: Serialize + Clone>(cmd: &str, data: T) -> String {
    format_log_complete(&format!("/api/ws {}", cmd), 200, Some(data.clone()));
    match to_string(&json!({ "cmd": cmd,  "status": 200, "data": data })) {
        Ok(msg) => msg,
        Err(error) => {
            error!("ERROR_CODE: 49696c3f-b4a2-444d-88b9-336f61c5e77b. Could not convert response to string: {}", error.to_string());
            exit(1);
        },
    }
}

pub fn ws_bad_request(cmd: &str, message: String) -> String {
    format_log_complete(&format!("/api/ws {}", cmd), 400, Some(message.clone()));
    match to_string(&json!({ "cmd": cmd,  "status": 400, "message": message })) {
        Ok(msg) => msg,
        Err(error) => {
            error!("ERROR_CODE: 0399a8fc-5b68-4d3f-bd09-238c0bdb0f8d. Could not convert response to string: {}", error.to_string());
            exit(1);
        },
    }
}

pub fn ws_conflict(cmd: &str, message: String) -> String {
    format_log_complete(&format!("/api/ws {}", cmd), 409, Some(message.clone()));
    match to_string(&json!({ "cmd": cmd,  "status": 409, "message": message })) {
        Ok(msg) => msg,
        Err(error) => {
            error!("ERROR_CODE: db29040b-3a86-44f4-8b30-b93ec9d569d3. Could not convert response to string: {}", error.to_string());
            exit(1);
        },
    }
}

pub fn ws_not_found(cmd: &str, message: String) -> String {
    format_log_complete(&format!("/api/ws {}", cmd), 404, Some(message.clone()));
    match to_string(&json!({ "cmd": cmd,  "status": 404, "message": message })) {
        Ok(msg) => msg,
        Err(error) => {
            error!("ERROR_CODE: 811db0e9-3fd7-4165-b5dd-fa5bebc8a73f. Could not convert response to string: {}", error.to_string());
            exit(1);
        },
    }
}

pub fn ws_reply_with<'a, T: Serialize + Clone>(
    ctx: &'a mut WebsocketContext<Websocket>,
    cmd: &'static str,
) -> impl FnMut(Reply<T>) + 'a {
    move |reply| {
        let message = match reply {
            Reply::Ok => ws_ok(cmd),
            Reply::OkWData(data) => ws_ok_w_data(cmd, data),
            Reply::BadRequest(message) => ws_bad_request(cmd, message),
            Reply::Conflict(message) => ws_conflict(cmd, message),
        };
        ctx.text(message)
    }
}

pub fn http_reply<T: Serialize + Clone>(route: &str, reply: Reply<T>) -> HttpResponse {
    match reply {
        Reply::Ok => http_ok(route),
        Reply::OkWData(data) => http_ok_w_data(route, data),
        Reply::BadRequest(message) => http_bad_request(route, message),
        Reply::Conflict(message) => http_conflict(route, message),
    }
}

pub fn http_ok(route: &str) -> HttpResponse {
    format_log_complete::<()>(route, 200, None);
    HttpResponse::Ok().finish()
}

pub fn http_ok_w_data<T: Serialize + Clone>(route: &str, data: T) -> HttpResponse {
    format_log_complete(route, 200, Some(data.clone()));
    HttpResponse::Ok().json(data)
}

pub fn http_bad_request(route: &str, message: String) -> HttpResponse {
    format_log_complete(route, 400, Some(message.clone()));
    HttpResponse::BadRequest()
        .content_type("text/plain")
        .body(message)
}

pub fn http_conflict(route: &str, message: String) -> HttpResponse {
    format_log_complete(route, 409, Some(message.clone()));
    HttpResponse::Conflict()
        .content_type("text/plain")
        .body(message)
}

pub fn prepend_data_str(message: String) -> String {
    format!("/data/{}", message)
}

pub fn append_null(message: String) -> String {
    format!("{} | null", message)
}

pub fn lock_or_exit<'a, T>(a: &'a Mutex<T>) -> MutexGuard<'a, T> {
    match a.lock() {
        Ok(a) => a,
        Err(error) => {
            error!("ERROR_CODE: 86238f73-20a5-47df-a7d0-60f991b02845. Could not get lock on mutext: {}", error.to_string());
            exit(1);
        }
    }
}

pub fn http_route_hit_log<T: Serialize>(route: &str, params: Option<T>) {
    if let Some(params) = params {
        if let Ok(params) = to_string(&params) {
            info!("{} {}", route, params);
        } else {
            error!("ERROR_CODE: 0c835bae-5421-4b09-9ef8-11254f6bace5. Could not format params for {}", route);
            exit(1);
        }
    } else {
        info!("{}", route);
    }
}

pub fn format_log_complete<T: Serialize>(route: &str, status_code: u32, return_body: Option<T>) {
    if let Some(return_body) = return_body {
        if let Ok(return_body) = to_string(&return_body) {
            info!("{} {} {}", route, status_code, return_body);
        } else {
            error!("ERROR_CODE: daaf2687-ae47-4a64-ae44-76ca7c89161b. Could not format return body for {}", route);
            exit(1);
        }
    } else {
        info!("{} {}", status_code, route);
    }
}
