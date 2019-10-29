use std::fmt::{Display, Formatter, Error};

#[derive(Hash, Clone, Debug, Eq, PartialEq)]
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

#[derive(Hash, Clone, Debug, Eq, PartialEq)]
pub struct ConnectionStatus {
    pub inode: usize,
    pub bytes_transferred: usize,
}


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