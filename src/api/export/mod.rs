use actix_web::{
    HttpResponse,
    web::{Data, Query}
};
use crate::AppData;
use msg_store::Uuid;
use serde::{
    Deserialize, 
    Serialize
};
use serde_json::to_string;
use std::{

    fs::OpenOptions,
    path::PathBuf,
    ops::Bound::Included,
    str::FromStr, io::Write
};

#[derive(Debug, Deserialize, Serialize)]
pub struct StoredPacket {
    pub uuid: String,
    pub msg: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    directory: String,
    priority: Option<i32>,
    range: Option<(Option<i32>, Option<i32>)>
}

// #[derive(Debug, Deserialize, Serialize)]
// #[serde(untagged)]
// pub enum Reply {
//     Ok { data: StoreData }
// }

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

        } else if let Some((start, end)) = info.range {

            let start = if let Some(start) = start {
                start
            } else {
                i32::MIN
            };

            let end = if let Some(end) = end {
                end
            } else {
                i32::MAX
            };

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

    let file_path = match PathBuf::from_str(&info.directory) {
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

        let msg_option = match store.get(Some(uuid), None) {
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
