use std::sync::mpsc::Receiver;

use crate::structs::connection::Connections;
use crate::structs::process::ProcessInfos;

pub type ConnectionsReceiver = Receiver<Connections>;
pub type ProcessesReceiver = single_value_channel::Receiver<Option<ProcessInfos>>;
pub type CaptureReceiver = single_value_channel::Receiver<Option<Connections>>;
