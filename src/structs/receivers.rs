use crate::structs::connection::Connections;
use crate::structs::process::ProcessInfos;

pub type ConnectionsReceiver = single_value_channel::Receiver<Option<Connections>>;
pub type ProcessesReceiver = single_value_channel::Receiver<Option<ProcessInfos>>;
pub type CaptureReceiver = single_value_channel::Receiver<Option<Connections>>;
