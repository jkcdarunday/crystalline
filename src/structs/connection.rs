use std::collections::HashMap;
use std::fmt::{Display, Error, Formatter};

use serde::ser;
use serde::ser::SerializeStruct;
use serde_derive::Serialize;

#[derive(Hash, Clone, Debug, Eq, PartialEq, Serialize)]
pub enum TransportType {
    Tcp,
    Udp,
}

#[derive(Hash, Clone, Debug, Eq, PartialEq)]
pub struct Connection {
    pub source_ip: Vec<u8>,
    pub source_port: u16,
    pub destination_ip: Vec<u8>,
    pub destination_port: u16,
    pub transport_type: TransportType,
}

#[derive(Hash, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ConnectionStatus {
    pub inode: usize,
    pub bytes_transferred: usize,
}

pub type ConnectionWithStatus = HashMap<Connection, ConnectionStatus>;

fn ip_to_string(ip: &Vec<u8>) -> String {
    let ip_string_array: Vec<String> = ip.iter().map(|num| num.to_string()).collect();

    ip_string_array.join(".")
}

impl Display for Connection {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}:{} - {}:{} {:?}",
               ip_to_string(&self.source_ip),
               self.source_port,
               ip_to_string(&self.destination_ip),
               self.destination_port,
               self.transport_type)
    }
}

impl serde::Serialize for Connection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: ser::Serializer, {
        let mut connection = serializer.serialize_struct("Connection", 5)?;
        connection.serialize_field("source_ip", &self.source_ip.iter().map(|ip| ip.to_string()).collect::<Vec<String>>().join("."))?;
        connection.serialize_field("destination_ip", &self.destination_ip.iter().map(|ip| ip.to_string()).collect::<Vec<String>>().join("."))?;
        connection.serialize_field("source_port", &self.source_port)?;
        connection.serialize_field("destination_port", &self.destination_port)?;
        connection.serialize_field("transport_type", &self.transport_type)?;
        connection.end()
    }
}
