use crate::{
    api::{
        get_required_priority, http_reply, update_config,
        // ws::{command::GROUP_DEFAULTS_DELETE, Websocket},
        // ws_reply_with,
        Reply, lock_or_exit, http_route_hit_log
    },
    config::GroupConfig,
    AppData,
};
use actix_web::{
    web::{Data, Query},
    HttpResponse,
};
use actix_web_actors::ws::WebsocketContext;
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::exit;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Info {
    priority: u32,
}

pub fn handle(data: Data<AppData>, info: Info) -> Reply<()> {
    {
        let mut store = lock_or_exit(&data.store);
        store.delete_group_defaults(info.priority);
    }
    {
        let mut config = lock_or_exit(&data.configuration);
        let groups = match &config.groups {
            Some(groups) => groups,
            None => {
                return Reply::Ok;
            }
        };
        let new_groups: Vec<GroupConfig> = groups
            .iter()
            .filter(|group| {
                if group.priority != info.priority {
                    true
                } else {
                    false
                }
            })
            .map(|group| group.clone())
            .collect();
        config.groups = Some(new_groups);
        if let Err(error) = update_config(&config, &data.configuration_path) {
            error!("ERROR_CODE: 11d80bf2-5b87-436f-9c26-914ca2718347. Could not update config file: {}", error);
            exit(1);
        }
    }
    Reply::Ok
}

const ROUTE: &'static str = "DEL /api/group-defaults";
pub fn http_handle(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    http_route_hit_log(ROUTE, Some(info.clone()));
    http_reply(ROUTE, handle(data, info.into_inner()))
}

// pub fn ws_handle(ctx: &mut WebsocketContext<Websocket>, data: Data<AppData>, info: Value) {
//     http_route_hit_log(GROUP_DEFAULTS_DELETE, Some(info.clone()));
//     let mut reply = ws_reply_with(ctx, GROUP_DEFAULTS_DELETE);
//     let priority = match get_required_priority(&info) {
//         Ok(priority) => priority,
//         Err(message) => return reply(Reply::BadRequest(message)),
//     };
//     reply(handle(data, Info { priority }));
// }
