use std::collections::HashSet;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use crate::structs::connection::{Connection, Connections};
use crate::structs::receivers::ConnectionsReceiver;

pub fn run(interval: u64) -> (JoinHandle<()>, ConnectionsReceiver) {
    let (receiver, updater) = single_value_channel::channel();

    let mut connections: HashSet<Connection> = HashSet::new();
    let handle = thread::spawn(move || {
        loop {
            let current_connections = get_tcp_connection_inodes();
            update_connections(&mut connections, &current_connections);
            prune_outdated_connections(&mut connections);

            let connections_arr: Vec<Connection> = connections.iter().cloned().collect();
            updater.update(Some(connections_arr)).unwrap();
            println!("Connection count: {}", connections.len());
            thread::sleep(Duration::from_millis(interval));
        }
    });

    (handle, receiver)
}

pub fn update_connections(
    connections: &mut HashSet<Connection>,
    current_connections: &Connections,
) {
    for connection in current_connections {
        let find_connection_result = connections.iter().find(|c| **c == *connection);
        if let Some(&ref found_connection) = find_connection_result {
            let mut new_connection = found_connection.clone();
            new_connection.last_seen = connection.last_seen;
            connections.replace(new_connection);
        } else {
            connections.insert(connection.clone());
        }
    }
}

pub fn prune_outdated_connections(connections: &mut HashSet<Connection>) {
    // Note: we can use drain_filter once it is stable
    let to_remove: Connections = connections.iter().filter(|c|
        c.last_seen.elapsed().unwrap_or(Duration::from_secs(0)).as_secs() > 60
    ).cloned().collect();
    for connection in to_remove {
        connections.remove(&connection);
    }

    connections.shrink_to_fit();
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
