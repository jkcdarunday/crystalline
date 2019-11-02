use std::collections::HashMap;
use std::sync::mpsc::Receiver;

use libc::pid_t;

use crate::structs::connection::Connections;

pub type ConnectionsReceiver = Receiver<Connections>;
pub type ProcessesReceiver = single_value_channel::Receiver<Option<HashMap<pid_t, Vec<u32>>>>;
pub type CaptureReceiver = single_value_channel::Receiver<Option<Connections>>;
