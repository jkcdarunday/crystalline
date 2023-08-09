use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::net::SocketAddr;
use std::time::SystemTime;

use libc::pid_t;
use procfs::net::{TcpNetEntry, UdpNetEntry};
use serde_derive::Serialize;

use crate::structs::process::ProcessInfos;

#[derive(Hash, Clone, Debug, Eq, PartialEq, Serialize)]
pub enum TransportType {
    Tcp,
    Udp,
}

#[derive(Clone, Debug, Serialize, Eq)]
pub struct Connection {
    pub source: SocketAddr,
    pub destination: SocketAddr,
    pub inode: u64,
    pub process_id: pid_t,
    pub transport_type: TransportType,
    pub bytes_uploaded: usize,
    pub bytes_downloaded: usize,
    pub first_seen: SystemTime,
    pub last_seen: SystemTime,
}

pub type Connections = Vec<Connection>;

impl From<TcpNetEntry> for Connection {
    fn from(entry: TcpNetEntry) -> Self {
        Connection {
            source: entry.local_address,
            destination: entry.remote_address,
            inode: entry.inode,
            process_id: 0,
            transport_type: TransportType::Tcp,
            bytes_uploaded: 0,
            bytes_downloaded: 0,
            first_seen: SystemTime::now(),
            last_seen: SystemTime::now(),
        }
    }
}

impl From<UdpNetEntry> for Connection {
    fn from(entry: UdpNetEntry) -> Self {
        Connection {
            source: entry.local_address,
            destination: entry.remote_address,
            inode: entry.inode,
            process_id: 0,
            transport_type: TransportType::Udp,
            bytes_uploaded: 0,
            bytes_downloaded: 0,
            first_seen: SystemTime::now(),
            last_seen: SystemTime::now(),
        }
    }
}

impl PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        self.transport_type == other.transport_type
            && ((self.source == other.source && self.destination == other.destination)
                || (self.source == other.destination && self.destination == other.source))
    }
}

impl Hash for Connection {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.transport_type.hash(state);
        if self.source < self.destination {
            self.source.hash(state);
            self.destination.hash(state);
        } else {
            self.destination.hash(state);
            self.source.hash(state);
        }
    }
}

impl Ord for Connection {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for Connection {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some((self.bytes_uploaded + self.bytes_uploaded).cmp(&(other.bytes_uploaded + other.bytes_downloaded)))
    }
}

impl Connection {
    pub fn bind_matching_process (&mut self, processes: &ProcessInfos) {
        for (process_id, process_info) in processes {
            if process_info.inodes.contains(&self.inode) {
                self.process_id = *process_id;
                break;
            }
        }
    }
}

//fn ip_to_string(ip: &Vec<u8>) -> String {
//    let ip_string_array: Vec<String> = ip.iter().map(|num| num.to_string()).collect();
//
//    ip_string_array.join(".")
//}
//
//impl Display for ConnectionEntry {
//    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
//        write!(f, "{}:{} - {}:{} {:?}",
//               ip_to_string(&self.source_ip),
//               self.source_port,
//               ip_to_string(&self.destination_ip),
//               self.destination_port,
//               self.transport_type)
//    }
//}
//
//impl serde::Serialize for ConnectionEntry {
//    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: ser::Serializer, {
//        let mut connection = serializer.serialize_struct("Connection", 5)?;
//        connection.serialize_field("source_ip", &self.source_ip.iter().map(|ip| ip.to_string()).collect::<Vec<String>>().join("."))?;
//        connection.serialize_field("destination_ip", &self.destination_ip.iter().map(|ip| ip.to_string()).collect::<Vec<String>>().join("."))?;
//        connection.serialize_field("source_port", &self.source_port)?;
//        connection.serialize_field("destination_port", &self.destination_port)?;
//        connection.serialize_field("transport_type", &self.transport_type)?;
//        connection.end()
//    }
//}
