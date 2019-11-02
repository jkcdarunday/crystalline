use std::net::SocketAddr;

use procfs::net::{TcpNetEntry, UdpNetEntry};
use serde_derive::Serialize;

#[derive(Hash, Clone, Debug, Eq, PartialEq, Serialize)]
pub enum TransportType {
    Tcp,
    Udp,
}

#[derive(Clone, Debug, Serialize)]
pub struct Connection {
    pub source: SocketAddr,
    pub destination: SocketAddr,
    pub inode: u32,
    pub transport_type: TransportType,
    pub bytes_uploaded: usize,
    pub bytes_downloaded: usize,
}

pub type Connections = Vec<Connection>;

impl From<TcpNetEntry> for Connection {
    fn from(entry: TcpNetEntry) -> Self {
        Connection {
            source: entry.local_address,
            destination: entry.remote_address,
            inode: entry.inode,
            transport_type: TransportType::Tcp,
            bytes_uploaded: 0,
            bytes_downloaded: 0,
        }
    }
}

impl From<UdpNetEntry> for Connection {
    fn from(entry: UdpNetEntry) -> Self {
        Connection {
            source: entry.local_address,
            destination: entry.remote_address,
            inode: entry.inode,
            transport_type: TransportType::Udp,
            bytes_uploaded: 0,
            bytes_downloaded: 0,
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
