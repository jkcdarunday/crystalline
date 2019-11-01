use std::collections::HashMap;
use std::sync::mpsc::Receiver;

use crate::structs::connection::{Connection, ConnectionWithStatus};

pub type ConnectionsReceiver = Receiver<HashMap<Connection, usize>>;
pub type ProcessesReceiver = single_value_channel::Receiver<Option<HashMap<usize, Vec<u64>>>>;
pub type CaptureReceiver = single_value_channel::Receiver<Option<ConnectionWithStatus>>;
