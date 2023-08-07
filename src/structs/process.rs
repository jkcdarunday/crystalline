use libc::pid_t;
use std::collections::HashMap;
use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;

#[derive(Clone, Debug)]
pub struct ProcessInfo {
    pub pid: pid_t,
    pub command: String,
    pub executable: String,
    pub inodes: Vec<u64>,
}

pub type ProcessInfos = HashMap<pid_t, ProcessInfo>;

impl Serialize for ProcessInfo {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
        S: Serializer {
        let mut process_info = serializer.serialize_struct("ProcessInfo", 4)?;

        process_info.serialize_field("pid", &self.pid)?;
        process_info.serialize_field("command", &self.command)?;
        process_info.serialize_field("executable", &self.executable)?;
        process_info.skip_field("inodes")?;

        process_info.end()
    }
}
