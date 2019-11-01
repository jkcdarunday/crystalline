use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::Duration;
use std::thread::JoinHandle;
use crate::structs::connection::{TransportType, Connection};

fn parse_hex_ipv4(ip: &str) -> Option<(Vec<u8>, u16)> {
    let mut split = ip.split(':');

    let mut ip_address = hex::decode(split.next()?).unwrap();
    ip_address.reverse();

    let port = hex::decode(split.next()?).unwrap();
    let port_number = port[0] as u16 * 256u16 + port[1] as u16;

    Some((ip_address, port_number))
}

pub fn run(interval: u64) -> (JoinHandle<()>, Receiver<HashMap<Connection, usize>>) {
    let (sender, receiver) = channel();

    let handle = thread::spawn(move || {
        loop {
            sender.send(get_tcp_connection_inodes().unwrap()).unwrap();
            thread::sleep(Duration::from_millis(interval));
        }
    });

    (handle, receiver)
}

pub fn get_tcp_connection_inodes() -> Option<HashMap<Connection, usize>> {
    let mut connection_inodes = HashMap::new();

    let file = match File::open("/proc/net/tcp") {
        Ok(handle) => handle,
        Err(_) => return None,
    };
    let buf_reader = BufReader::new(file);

    let mut lines = buf_reader.lines();
    lines.next(); // throw away first line
    for line_result in lines {
        let line = line_result.unwrap();

        let mut line_split = line.trim().split_ascii_whitespace();
        let (source, destination, inode) = (
            line_split.nth(1)?,
            line_split.nth(0)?,
            line_split.nth(6)?.parse::<usize>().unwrap()
        );

        let (source_ip, source_port) = parse_hex_ipv4(source)?;
        let (destination_ip, destination_port) = parse_hex_ipv4(destination)?;

        let connection = Connection {
            source_ip,
            source_port,
            destination_ip,
            destination_port,
            transport_type: TransportType::Tcp,
        };
        connection_inodes.insert(connection, inode);
    }


    Some(connection_inodes)
}
