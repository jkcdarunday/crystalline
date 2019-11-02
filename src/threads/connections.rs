use std::sync::mpsc::channel;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use crate::structs::connection::{Connection, Connections};
use crate::structs::receivers::ConnectionsReceiver;

pub fn run(interval: u64) -> (JoinHandle<()>, ConnectionsReceiver) {
    let (sender, receiver) = channel();

    let handle = thread::spawn(move || {
        loop {
            sender.send(get_tcp_connection_inodes()).unwrap();
            thread::sleep(Duration::from_millis(interval));
        }
    });

    (handle, receiver)
}

pub fn get_tcp_connection_inodes() -> Connections {
    let mut connections = Connections::new();

    for entry in procfs::net::tcp().unwrap() {
        connections.push(Connection::from(entry));
    }

    for entry in procfs::net::tcp6().unwrap() {
        connections.push(Connection::from(entry));
    }

    for entry in procfs::net::udp().unwrap() {
        connections.push(Connection::from(entry));
    }

    for entry in procfs::net::udp6().unwrap() {
        connections.push(Connection::from(entry));
    }

    connections
}
