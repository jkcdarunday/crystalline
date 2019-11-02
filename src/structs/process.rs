use libc::pid_t;
use serde_derive::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct ProcessInfo {
    pub pid: pid_t,
    pub command: String,
    pub executable: String,
    pub inodes: Vec<u32>,
}

pub type ProcessInfos = Vec<ProcessInfo>;
