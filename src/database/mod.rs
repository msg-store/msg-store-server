pub mod leveldb {
    use bincode::{
        deserialize,
        serialize
    };
    use crate::{
        StoreConfig,
        fmt_result
    };
    use leveldb::{
        database::{
            Database
        },
        iterator::Iterable,
        options::{
            Options,
            ReadOptions,
            WriteOptions
        },
        kv::KV
    };
    use msg_store::store::{
        GroupDefaults,
        GroupId,
        Group,
        MsgId,
        MsgByteSize,
        Store
    };
    use serde::{
        Deserialize,
        Serialize
    };
    use std::{
        collections::{
            BTreeMap
        },
        fs::create_dir_all,
        path::Path
    };

    #[derive(Debug, Deserialize, Serialize)]
    pub struct MsgData {
        pub priority: GroupId,
        pub msg_byte_size: MsgByteSize
    }

    pub struct Db {
        msgs: Database<MsgId>,
        msg_data: Database<MsgId>
    }

    impl Db {
        pub fn open(dir: &Path) -> Result<Db, String> {

            create_dir_all(&dir).expect("Could not create db location dir.");

            let mut msgs_path = dir.to_path_buf();
            msgs_path.push("msgs");
            let msgs_path = msgs_path.as_path();

            let mut msg_data_path = dir.to_path_buf();
            msg_data_path.push("msg_data");
            let msg_data_path = msg_data_path.as_path();

            let mut msgs_options = Options::new();
            msgs_options.create_if_missing = true;

            let mut msg_data_options = Options::new();
            msg_data_options.create_if_missing = true;

            let msgs = fmt_result!(Database::open(msgs_path, msgs_options))?;
            let msg_data = fmt_result!(Database::open(Path::new(msg_data_path), msg_data_options))?;
            
            Ok(Db {
                msgs,
                msg_data
            })

        }
        pub fn pull(&mut self) -> Result<(BTreeMap<MsgId, GroupId>, BTreeMap<GroupId, Group>), String> {
            let options = ReadOptions::new();
            let mut id_to_priority_map: BTreeMap<MsgId, GroupId> = BTreeMap::new();
            let mut groups: BTreeMap<GroupId, Group> = BTreeMap::new();
            for (key, value) in self.msg_data.iter(options) {
                let data: MsgData = fmt_result!(deserialize(&value))?;
                let mut group = match groups.remove(&data.priority) {
                    Some(group) => group,
                    None => Group {
                        max_byte_size: None,
                        byte_size: 0,
                        msgs_map: BTreeMap::new()
                    }
                };                
                group.msgs_map.insert(key, data.msg_byte_size);
                group.byte_size += data.msg_byte_size;
                groups.insert(data.priority, group);
                id_to_priority_map.insert(key, data.priority);
            }
            Ok((id_to_priority_map, groups))
        }
        pub fn put(&mut self, id: &MsgId, msg_data: &MsgData, msg: &str) -> Result<(), String> {
            let msg_data_write_opts = WriteOptions::new();
            let msg_write_opts = WriteOptions::new();
            let serialized_data = fmt_result!(serialize(msg_data))?;
            let serialized_msg = fmt_result!(serialize(msg))?;
            fmt_result!(self.msg_data.put(msg_data_write_opts, id, &serialized_data))?;
            fmt_result!(self.msgs.put(msg_write_opts, id, &serialized_msg))?;
            Ok(())
        }
        pub fn get(&mut self, id: &MsgId) -> Result<Option<String>, String> {
            let read_opts = ReadOptions::new();
            let msg_opt = fmt_result!(self.msgs.get(read_opts, id))?;
            let serialized_msg = match msg_opt {
                Some(serialized_msg) => serialized_msg,
                None => {
                    return Ok(None);
                }
            };
            let msg: String = fmt_result!(deserialize(&serialized_msg))?;
            Ok(Some(msg))
        }
        pub fn delete(&mut self, id: &MsgId) -> Result<(), String> {
            let write_opts = WriteOptions::new();
            fmt_result!(self.msg_data.delete(write_opts, id))?;
            let write_opts = WriteOptions::new();
            fmt_result!(self.msgs.delete(write_opts, id))?;
            Ok(())
        }
        pub fn update(&mut self, id: &MsgId, msg_data: &MsgData) -> Result<(), String> {
            let msg_data_write_opts = WriteOptions::new();
            let serialized_data = fmt_result!(serialize(msg_data))?;
            fmt_result!(self.msg_data.put(msg_data_write_opts, id, &serialized_data))?;
            Ok(())
        }
        pub fn setup(&mut self, store: &mut Store, config: &StoreConfig) -> Result<(), String> {
            if let Some(store_max_byte_size) = config.max_byte_size {
                store.max_byte_size = Some(store_max_byte_size);
            }
        
            if let Some(config_group_defaults) = &config.groups {
                for group_config in config_group_defaults.iter() {
                    let defaults = GroupDefaults { 
                        max_byte_size: group_config.max_byte_size
                    };
                    store.group_defaults.insert(group_config.priority, defaults);
                }
            }
        
            let (id_to_priority, mut groups) = self.pull().unwrap();
            store.id_to_group_map = id_to_priority;
            for (id, mut group) in groups.iter_mut() {
                if let Some(defaults) = store.group_defaults.get(&id) {
                    group.max_byte_size = defaults.max_byte_size;
                }
                store.byte_size += group.byte_size;
            }
            store.groups_map = groups;
        
            let last_id = match store.id_to_group_map.keys().rev().next() {
                Some(id) => *id,
                None => 0
            };
            store.next_id = last_id;
            Ok(())
        }
    }

}