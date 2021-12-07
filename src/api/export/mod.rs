use actix_web::{
    HttpResponse,
    web::{Data, Query}
};
use crate::AppData;
use msg_store::{GetOptions, Uuid};
use serde::{
    Deserialize, 
    Serialize
};
use serde_json::to_string;
use std::{
    fs::OpenOptions,
    io::Write,
    ops::Bound::Included,
    path::PathBuf,
    str::FromStr,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct StoredPacket {
    pub uuid: String,
    pub msg: String
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    output: String,
    priority: Option<u32>,
    range_start: Option<u32>,
    range_end: Option<u32>
}

pub fn get(data: Data<AppData>, info: Query<Info>) -> HttpResponse {    

    let list = {

        let store = match data.store.try_lock() {
            Ok(store) => store,
            Err(_error) => {
                return HttpResponse::InternalServerError().finish();
            }
        };
    
        let list: Vec<Uuid> = if let Some(priority) = info.priority {

            if let Some(group) = store.groups_map.get(&priority) {
                group.msgs_map.keys().map(|uuid| uuid.clone()).collect::<Vec<Uuid>>()            
            } else {
                vec![]
            }

        } else if let Some(start) = info.range_start {



            let end = if let Some(end) = info.range_end {
                end
            } else {
                u32::MAX
            };

            let mut list = vec![];
            for (_priority, group) in store.groups_map.range((Included(&start), Included(&end))) {
                list.append(&mut group.msgs_map.keys().map(|uuid| uuid.clone()).collect::<Vec<Uuid>>())
            }

            list

        } else if let Some(end) = info.range_end {

            let start = u32::MIN;

            let mut list = vec![];
            for (_priority, group) in store.groups_map.range((Included(&start), Included(&end))) {
                list.append(&mut group.msgs_map.keys().map(|uuid| uuid.clone()).collect::<Vec<Uuid>>())
            }

            list
            
        } else {

            let mut list = vec![];
            for group in store.groups_map.values() {
                list.append(&mut group.msgs_map.keys().map(|uuid| uuid.clone()).collect::<Vec<Uuid>>())
            }

            list

        };

        list

    };

    let file_path = match PathBuf::from_str(&info.output) {
        Ok(dir_path) => dir_path,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    
    let mut file = match OpenOptions::new().append(true).create_new(true).open(file_path) {
        Ok(file) => file,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };

    for uuid in list {

        let mut store = match data.store.try_lock() {
            Ok(store) => store,
            Err(_error) => {
                return HttpResponse::InternalServerError().finish();
            }
        };

        let msg_option = match store.get(GetOptions::default().uuid(uuid)) {
            Ok(msg) => msg,
            Err(_error) => {
                return HttpResponse::InternalServerError().finish();
            }
        };

        let stored_packet = if let Some(stored_packet) = msg_option {
            stored_packet
        } else {
            continue;
        };

        let transformed_stored_packet = StoredPacket {
            uuid: stored_packet.uuid.to_string(),
            msg: stored_packet.msg
        };

        
        let mut packet_string = match to_string(&transformed_stored_packet) {
            Ok(packet_string) => packet_string,
            Err(_error) => {
                return HttpResponse::InternalServerError().finish();
            }
        };
        packet_string.push_str("\n");

        if let Err(_error) = file.write_all(packet_string.as_bytes()) {
            return HttpResponse::InternalServerError().finish();
        };

        if let Err(_error) = store.del(&uuid) {
            return HttpResponse::InternalServerError().finish();
        }

    }

    HttpResponse::Ok().finish()
}
