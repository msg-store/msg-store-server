use crate::{
    api::{
        // get_require_msg, get_required_priority, 
        http_reply,
        // ws::{command::MSG_POST, Websocket},
        // ws_reply_with, 
        Reply, lock_or_exit, 
        http_route_hit_log,
        lower::{
            Database,
            FileList,
            Store
        }
    },
    AppData,
};
use actix_web::{
    web::{
        Data, 
        // Json, 
        Payload, BytesMut
    },
    HttpResponse,
};

use futures::StreamExt;
use log::{
    error, 
    debug
};
use msg_store::errors::Error;
use serde::{Deserialize, Serialize};
// use serde_json::Value;
use std::{
    collections::BTreeMap,
    process::exit,
    fs::{File, remove_file},
    io::Write,
    sync::Mutex
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Body {
    priority: u32,
    msg: String,
}

// NEW BODY => PRIORITY=1 MSG=my really long msg ...

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReturnBody {
    uuid: String,
}

pub async fn handle(
    store: Store,
    file_list: Option<FileList>, mut payload: Payload) -> Reply<ReturnBody> {

    let mut metadata_string = String::new();
    let mut msg_chunk = BytesMut::new();
    let mut metadata: BTreeMap<String, String> = BTreeMap::new();
    let mut save_to_file = false;

    while let Some(chunk) = payload.next().await {
        let chunk = match chunk {
            Ok(chunk) => chunk,
            Err(error) => {
                error!("ERROR_CODE: bba7f467-7d6a-46a9-8fdd-1935c11dbcc5. Could parse mutlipart field: {}", error.to_string());
                exit(1);
            }
        };
        let mut chunk_string = match String::from_utf8(chunk.to_vec()) {
            Ok(chunk_string) => chunk_string,
            Err(error) => {
                error!("ERROR_CODE: 137fc6cd-1963-4601-9fc1-e78f73dab202. Could parse chunk: {}", error.to_string());
                exit(1);
            }
        };
        chunk_string = chunk_string.trim_start().to_string();
        // debug!("recieved chunk: {}", chunk_string);
        metadata_string.push_str(&chunk_string);
        if metadata_string.contains("?") {
            // debug!("msg start found!");
            if metadata_string.len() == 0 {
                return Reply::BadRequest("Missing headers".to_string());
            }
            match metadata_string.split_once("?") {
                Some((metadata_section, msg_section)) => {
                    for pair in metadata_section.to_string().split("&").into_iter() {
                        let kv = pair.trim_end().trim_start().split("=").map(|txt| txt.to_string()).collect::<Vec<String>>();
                        let k = match kv.get(0) {
                            Some(k) => k.clone(),
                            None => {
                                return Reply::BadRequest("Malformed header".to_string());
                            }
                        };
                        let v = match kv.get(1) {
                            Some(v) => v.clone(),
                            None => {
                                return Reply::BadRequest("Malformed header".to_string());
                            }
                        };
                        metadata.insert(k, v);
                    };
                    msg_chunk.extend_from_slice(msg_section.as_bytes());
                },
                None => {
                    error!("ERROR_CODE: 82e215db-1067-4ccd-bc71-e818ec60857e. Could parse chunk");
                    exit(1);
                }
            }
            if let Some(save_to_file_value) = metadata.remove("saveToFile") {
                if save_to_file_value.to_lowercase() == "true" {
                    if let None = file_list {
                        return Reply::BadRequest("Store does not allow files to be saved.".to_string());
                    }
                    save_to_file = true;
                }
            }
            break;
        }
    }

    let priority: u32 = match metadata.remove("priority") {
        Some(priority) => match priority.parse() {
            Ok(priority) => priority,
            Err(_error) => { return Reply::BadRequest("Invalid priority".to_string()) }
        },
        None => { return Reply::BadRequest("Missing priority field".to_string()) }
    };

    // debug!("text: {}", buffer_str);
    let (msg_byte_size, msg) = {
        if save_to_file == true {
            if let Some(byte_size_override_str) = metadata.get("byteSizeOverride") {
                let msg_byte_size = match byte_size_override_str.parse::<u32>() {
                    Ok(byte_size_override) => byte_size_override,
                    Err(_error) => { return Reply::BadRequest("Invalid byteSizeOverride".to_string())}
                };
                let msg_parse = metadata
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<String>>()
                    .join("&");
                (msg_byte_size, msg_parse)
            } else {
                return Reply::BadRequest("Missing byteSizeOverride".to_string());
            }
        } else {
            while let Some(chunk) = payload.next().await {
                let chunk = match chunk {
                    Ok(chunk) => chunk,
                    Err(error) => {
                        error!("ERROR_CODE: ac566987-87f3-4a59-becc-ef6f264b826b. Could parse mutlipart field: {}", error.to_string());
                        exit(1);
                    }
                };
                msg_chunk.extend_from_slice(&chunk);
            }
            match String::from_utf8(msg_chunk.to_vec()) {
                Ok(msg) => (msg.len() as u32, msg),
                Err(error) => {
                    error!("ERROR_CODE: 7e1f5600-a2f5-45fa-9d8f-6c62ca2e2f14. Could not parse msg from buffer: {}", error.to_string());
                    exit(1);
                }
            }
        }
    };
    
    // debug!("priority: {}, msg_byte_size: {}, save_to_file: {}, msg: {}", priority, msg_byte_size, save_to_file, msg);

    // let msg_byte_size = msg.len() as u32;
    let add_result = {
        let mut store = lock_or_exit(&data.store);      
        match store.add(priority, msg_byte_size) {
            Ok(add_result) => add_result,
            Err(error) => match error {
                Error::ExceedesStoreMax => {
                    return Reply::Conflict(
                        "Message byte size exceeds the max byte size limit allowed by the store"
                            .to_string(),
                    );
                }
                Error::ExceedesGroupMax => {
                    return Reply::Conflict(
                        "Message byte size exceeds the max byte size limit allowed by the group"
                            .to_string(),
                    );
                }
                Error::LacksPriority => {
                    return Reply::Conflict(
                        "The store has reached max capcity and could not accept message".to_string(),
                    );
                }
                error => {
                    error!("ERROR_CODE: c911f827-35ec-42aa-8ca3-2b10b68209c9. {}", error.to_string());
                    exit(1)
                },
            },
        }
    };
    
    // remove msgs from db
    let mut deleted_count = 0;
    for uuid in add_result.msgs_removed.into_iter() {
        {
            let mut db = lock_or_exit(&data.db);
            if let Err(error) = db.del(uuid) {
                error!("ERROR_CODE: 0753a0a2-5436-44e1-bb05-6e81193ad9e7. Could not remove msg from database: {}", error);
                exit(1);
            }
        }
        {
            let file_list = match &data.file_list {
                Some(file_list) => file_list,
                None => {
                    error!("ERROR_CODE: 95483642-ff35-48e4-89b8-6b4d8d44b246. File list could not be found.");
                    exit(1);
                }
            };
            let mut file_list = lock_or_exit(&file_list);
            if file_list.remove(&uuid) {
                if let Err(error) = remove_file(format!("/tmp/msg-store-file-items/{}", uuid.to_string())) {
                    error!("ERROR_CODE: a86b06c9-fd6a-4bd9-b30b-3ab4895af8e0. Could not remove file: {}", error.to_string());
                    exit(1);
                }
            }
        }
        deleted_count += 1;
    }
    {
        let mut stats = lock_or_exit(&data.stats);
        stats.deleted += deleted_count;
        stats.inserted += 1;
    }
    {
        if save_to_file {
            {
                let file_list = match &data.file_list {
                    Some(file_list) => file_list,
                    None => {
                        error!("ERROR_CODE: 4914b57f-0581-451d-996b-88e5953fa793. File list could not be found.");
                        exit(1);
                    }
                };
                let mut file_list = lock_or_exit(&file_list);
                file_list.insert(add_result.uuid.clone());
            }
            // TODO: change file save location
            let mut file = match File::create(format!("/tmp/msg-store-file-items/{}", add_result.uuid.to_string())) {
                Ok(file) => file,
                Err(error) => {
                    error!("ERROR_CODE: 0df10fb3-058f-4e4c-936e-ab8d8f2865e1. Could not create file: {}", error);
                    exit(1);
                }
            };
            if let Err(error) = file.write(&msg_chunk.to_vec()) {
                error!("ERROR_CODE: ead714f3-9217-4d4d-bcf0-dc592421e429. Could not write to file: {}", error);
                exit(1);
            };
            while let Some(chunk) = payload.next().await {
                let chunk = match chunk {
                    Ok(chunk) => chunk,
                    Err(error) => {
                        error!("ERROR_CODE: ac566987-87f3-4a59-becc-ef6f264b826b. Could parse mutlipart field: {}", error.to_string());
                        exit(1);
                    }
                };
                if let Err(error) = file.write(&chunk) {
                    error!("ERROR_CODE: 0e15385e-d6c5-4eee-bffa-b7fb79916424. Could not write to file: {}", error);
                    exit(1);
                };
            }
            let mut db = lock_or_exit(&data.db);
            if let Err(error) = db.add(add_result.uuid, msg, msg_byte_size) {
                error!("ERROR_CODE: f106eed6-1c47-4437-b9c3-082a4c5393af. Could not add msg to database: {}", error);
                exit(1);
            }
        } else {
            let mut db = lock_or_exit(&data.db);
            if let Err(error) = db.add(add_result.uuid, msg, msg_byte_size) {
                error!("ERROR_CODE: f106eed6-1c47-4437-b9c3-082a4c5393af. Could not add msg to database: {}", error);
                exit(1);
            }
        }
    }
    Reply::OkWData(ReturnBody {
        uuid: add_result.uuid.to_string(),
    })
}
