use std::collections::HashMap;
use std::sync::mpsc::Receiver;

use libc::pid_t;

use crate::structs::connection::{Connection, ConnectionWithStatus};

pub type ConnectionsReceiver = Receiver<HashMap<Connection, usize>>;
pub type ProcessesReceiver = single_value_channel::Receiver<Option<HashMap<pid_t, Vec<u32>>>>;
pub type CaptureReceiver = single_value_channel::Receiver<Option<ConnectionWithStatus>>;
