use crate::{
    api::{
        get_optional_number, get_optional_string, get_required_string, http_route_hit_log, format_log_complete, http_reply,
        ws::command::EXPORT
    },
    AppData,
};
use actix_web::{
    web::{Data, Query},
    HttpResponse,
};
use actix_web_actors::ws::WebsocketContext;
use log::error;
use msg_store_db_plugin::Db;
use msg_store_plugin_leveldb::Leveldb;
use serde::{Deserialize, Serialize};
use serde_json::{to_string, Value};
use std::{
    fs::{OpenOptions, copy, remove_file},
    io::Write,
    path::PathBuf, 
    process::exit, 
    str::FromStr
};

// TODO: FIX: Export only exports msgs stored in the database, not the filesytem

use super::{
    append_null, prepend_data_str,
    ws::Websocket,
    ws_reply_with, Reply, lock_or_exit, FileManager,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct StoredPacket {
    pub uuid: String,
    pub msg: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    output_directory: String, // directory
    // priority: Option<u32>,
    // range_start: Option<String>,
    // range_end: Option<String>,
}

pub fn get_info(value: &Value) -> Result<Info, String> {
    let output_directory_option = match get_optional_string(&value, "outputDirectory") {
        Ok(output_directory_option) => output_directory_option,
        Err(message) => { return Err(prepend_data_str(message)) }
    };
    if let Some(output_directory) = output_directory_option {
        return Ok(Info { output_directory })
    }
    let output_directory = match get_required_string(&value, "output_irectory") {
        Ok(output_directory_option) => output_directory_option,
        Err(message) => { return Err(prepend_data_str(message)) }
    };
    return Ok(Info { output_directory })
}

pub fn handle(data: Data<AppData>, info: Info) -> Reply<()> {

    let max_count = {
        let store = lock_or_exit(&data.store);
        store.id_to_group_map.len()
    };

    let database = {
        let configuration = lock_or_exit(&data.configuration);
        if let Some(database) = &configuration.database {
            let lower_case = database.to_ascii_lowercase();
            if lower_case == "mem" || lower_case == "memory" {
                "mem"
            } else if lower_case == "leveldb" {
                "leveldb"
            } else {
                error!("ERROR_CODE: 5a586716-4d4f-4569-834c-601954c69202. Could not get database option.");
                exit(1);
            }
        } else {
            "mem"
        }
    };
    let deleted_count = {
        let mut deleted_count = 0;
        if database == "mem" {
            // convert output directory to pathbuf
            let dir_path = match PathBuf::from_str(&info.output_directory) {
                Ok(dir_path) => dir_path,
                Err(error) => {
                    error!("ERROR_CODE: ab65abce-5a25-415b-991a-7a540242d185. Could not parse file path: {}", error.to_string());
                    exit(1);
                }
            };
            // get the default file name
            let mut file_path = dir_path.to_path_buf();
            file_path.push("msg-store-backup.txt");
            // check if it exits
            if file_path.exists() {
                // if it exists, then append a number to the path and check if it too exits.
                // repeat until a non-existing path is found
                let mut count = 1;
                loop {
                    file_path = dir_path.to_path_buf();
                    file_path.push(format!("msg-store-backup-{}.txt", count));
                    if !file_path.exists() {
                        break;
                    }
                    count += 1;
                }
            }
            // open the file
            let mut file = match OpenOptions::new()
                .append(true)
                .create_new(true)
                .open(file_path)
            {
                Ok(file) => file,
                Err(error) => {
                    error!("ERROR_CODE: 5d935776-a61f-4b07-9a0f-0d4c7e68280e. Could not open file: {}", error.to_string());
                    exit(1);
                }
            };
            // get the number of messages to export
            for _i in 0..max_count {
                // get a uuid
                let uuid = {
                    let store = lock_or_exit(&data.store);
                    match store.get(None, None, false) {
                        Ok(uuid) => uuid,
                        Err(error) => {
                            error!("ERROR_CODE: eed54108-ed28-4809-97dc-3559930a1a4d. Could not get data from store: {}", error.to_string());
                            exit(1);
                        }
                    }            
                };
                // if no uuid is found then break the loop
                let uuid = if let Some(uuid) = uuid {
                    uuid
                } else {
                    break;
                };
                // get the message from the database    
                let msg = {
                    let mut db = lock_or_exit(&data.db);
                    match db.get(uuid.clone()) {
                        Ok(msg) => msg,
                        Err(error) => {
                            error!("ERROR_CODE: 244c3834-6d38-40f4-901b-58e7444a091a. Could not get msg from database: {}", error.to_string());
                            exit(1);
                        }
                    }
                };
                // convert the message to string (for some reason? This may be needed.)
                let msg = match String::from_utf8(msg.to_vec()) {
                    Ok(msg) => msg,
                    Err(_error) => {
                        error!("ERROR_CODE: 7862fbdf-0bdb-45ae-b3fd-71a053102451. Could not get msg from bytes");
                        exit(1);
                    }
                };
                // package up the data
                let transformed_stored_packet = StoredPacket {
                    uuid: uuid.to_string(),
                    msg,
                };
                // convert to string again? This also may not be needed.
                let mut packet_string = match to_string(&transformed_stored_packet) {
                    Ok(packet_string) => packet_string,
                    Err(error) => {
                        error!("ERROR_CODE: 175f94cb-3553-461d-9447-7dd3e3ff9b87. Could not convert data to string: {}", error.to_string());
                        exit(1);
                    }
                };
                packet_string.push_str("\n");
                if let Err(error) = file.write_all(packet_string.as_bytes()) {
                    error!("ERROR_CODE: 7471d35e-a5a2-4bd6-bab1-6017092d97b5. Could not write to file: {}", error.to_string());
                    exit(1);
                };
                // remove message from store
                {
                    let mut store = lock_or_exit(&data.store);
                    if let Err(error) = store.del(uuid.clone()) {
                        error!("ERROR_CODE: c70634e2-f090-43b1-883a-ccf8e62fbc30. Could not get mutex lock on store: {}", error.to_string());
                        exit(1);
                    }
                }
                // remove message from database
                {
                    let mut db = lock_or_exit(&data.db);
                    if let Err(error) = db.del(uuid) {
                        error!("ERROR_CODE: 4a25b256-753c-43c6-b564-35448d5ad9de. Could not get mutex lock on database: {}", error.to_string());
                        exit(1);
                    }
                }
                deleted_count += 1;
            }
        } else if database == "leveldb" {
            // convert the string into a pathbuf
            let export_dir_path = match PathBuf::from_str(&info.output_directory) {
                Ok(export_path) => export_path,
                Err(error) => {
                    return Reply::BadRequest(format!("The export path give could not be derived: {}", error.to_string()))
                }
            };
            // create a new path
            let mut backup_path = export_dir_path.to_path_buf();
            backup_path.push("msg-store-backup");
            // open a new file manager for the backup if one already exists on the server
            // this is so that we can we its boilerplate code for setting up the directory
            let file_manager_backup = {
                // check if the one currently exits
                if data.file_manager.is_some() {
                    // create a new one
                    let mut file_storage_backup_path = backup_path.to_path_buf();
                    file_storage_backup_path.push("file-storage");
                    Some(FileManager::new(&file_storage_backup_path))
                } else {
                    // do not create one
                    None
                }
            };
            // check if the path exists
            if backup_path.exists() {
                // if it exists, then append a number to the path and check if it too exits.
                // repeat until a non-existing path is found
                let mut count = 1;
                loop {
                    backup_path = export_dir_path.to_path_buf();
                    backup_path.push(format!("msg-store-backup-{}", count));
                    if !backup_path.exists() {
                        break;
                    }
                    count += 1;
                }
            }
            // get the leveldb path
            let mut leveldb_path = backup_path.to_path_buf();
            leveldb_path.push("leveldb");
            // open the leveldb instance
            let mut leveldb_backup = match Leveldb::new(&leveldb_path) {
                Ok(leveldb) => leveldb,
                Err(error) => {
                    error!("ERROR_CODE: 2a1cdb3f-a7f9-426a-a5f3-46ed6c534ad2. Could not create leveldb backup: {}", error);
                    exit(1);
                }
            };
            for _ in 0..max_count {
                let store = lock_or_exit(&data.store);
                let mut leveldb = lock_or_exit(&data.db);
                let uuid = match store.get(None, None, false) {
                    Ok(uuid) => uuid,
                    Err(error) => {
                        error!("ERROR_CODE: 02a2de7e-7c96-4afb-b0ca-44b04eefafe2. Could not get uuid from store: {}", error.to_string());
                        exit(1);
                    }
                };
                let uuid = match uuid {
                    Some(uuid) => uuid,
                    None => { break }
                };
                let msg = match leveldb.get(uuid.clone()) {
                    Ok(msg) => msg,
                    Err(error) => {
                        error!("ERROR_CODE: af2077be-050b-4e9c-9dbe-457c789115c1. Could not get msg from database: {}", error);
                        exit(1);
                    }
                };                
                let msg_byte_size = msg.len() as u32;

                // file storage section
                let mut src_file_path: Option<PathBuf> = None;
                let mut dest_file_path: Option<PathBuf> = None;
                // check if a file manager exists
                // TODO: Optimise to remove this check in every loop iteration
                if let Some(file_manager) = &data.file_manager {
                    {
                        let file_manager = lock_or_exit(&file_manager);
                        // check to see if the message is a file by looking up the file manager index
                        if file_manager.file_list.contains(&uuid) {
                            // recheck if the filemanager backup exists
                            // TODO: Optimise to remove this recheck in every loop iteration
                            if let Some(file_manager_backup) = &file_manager_backup {
                                // get the src file path
                                let mut file_path = file_manager.file_storage_path.to_path_buf();
                                file_path.push(uuid.clone().to_string());
                                // get the dest file path
                                let mut new_file_path = file_manager_backup.file_storage_path.to_path_buf();
                                new_file_path.push(uuid.clone().to_string());
                                // copy the contents
                                if let Err(error) = copy(&file_path, &new_file_path) {
                                    error!("ERROR_CODE: b5e42ab3-1530-49ce-b724-86f288e0b36a. Could not make backup copy of file: {}", error.to_string());
                                    exit(1);
                                };
                                // keep the file paths onhand incase it needs to be reverted after an error
                                src_file_path = Some(file_path);
                                dest_file_path = Some(new_file_path);
                            } else {
                                error!("ERROR_CODE: f32ebe39-11a8-47f6-976d-753c436e0fba. Could not find file manager backup.");
                                exit(1)
                            }
                        }
                    }
                    // remove the file from the index
                    FileManager::del(file_manager, uuid.clone())
                }
                // add the data to the leveldb backup
                // if it errors then copy the destination file back to the source
                // dont exit until on error handling has finished
                if let Err(error) = leveldb_backup.add(uuid, msg, msg_byte_size) {
                    error!("ERROR_CODE: 86fccd47-c4b9-4516-b292-dd1261213910. Could not add msg to backup: {}", error);
                    if let Some(src_file_path) = src_file_path {
                        if let Some(dest_file_path) = dest_file_path {
                            if let Err(error) = copy(&dest_file_path, &src_file_path) {
                                error!("ERROR_CODE: 7b3b3493-620e-4a73-9d58-622cef9ea714. Could not revert fs changes: {}", error.to_string());
                            };
                            if let Err(error) = remove_file(dest_file_path) {
                                error!("ERROR_CODE: 78c08983-ff20-4cfe-9420-489334406ca6. Could not remove destination file after error: {}", error.to_string());
                            }
                        }
                    }
                    exit(1);
                }
                // update deleted count
                deleted_count += 1;

            }
        } else {
            error!("ERROR_CODE: eae7b453-9532-47b0-b73b-01af6e4dddfc. Could not get database option.");
            exit(1);
        }
        deleted_count
    };
    // update stats
    {
        let mut stats = lock_or_exit(&data.stats);
        stats.deleted += deleted_count;
    }    
    Reply::Ok
}

const ROUTE: &'static str = "/api/export";

pub fn http_handle(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    http_route_hit_log(ROUTE, Some(info.clone()));
    http_reply(ROUTE, handle(data, info.into_inner()))
}

pub fn ws_handle(ctx: &mut WebsocketContext<Websocket>, data: Data<AppData>, info: Value) {
    http_route_hit_log(EXPORT, Some(info.clone()));
    let mut reply = ws_reply_with(ctx, EXPORT);
    let info = match get_info(&info) {
        Ok(info) => info,
        Err(message) => {
            format_log_complete::<()>(ROUTE, 400, None);
            return reply(Reply::BadRequest(message));
        }
    };
    reply(handle(data, info));
}
