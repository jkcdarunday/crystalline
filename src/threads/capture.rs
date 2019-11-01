use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::thread;
use std::thread::JoinHandle;

use etherparse::InternetSlice::Ipv4;
use etherparse::InternetSlice::Ipv6;
use etherparse::SlicedPacket;
use etherparse::TransportSlice::{Tcp, Udp};
use pcap::{Device, Packet};
use single_value_channel;

use crate::structs::connection::{Connection, ConnectionStatus, ConnectionWithStatus, TransportType};
use crate::structs::receivers::{CaptureReceiver, ConnectionsReceiver, ProcessesReceiver};
use crate::threads;

pub fn run(connections_thread: ConnectionsReceiver, mut processes_thread: ProcessesReceiver) -> (JoinHandle<()>, CaptureReceiver) {
//    let (sender, receiver) = channel();
    let (receiver, updater) = single_value_channel::channel();

    let handle = thread::spawn(move || {
        let _process_inodes = match processes_thread.latest() {
            Some(inodes) => inodes.clone(),
            None => threads::processes::get_inodes_per_process()
        };
        let mut connections = HashMap::<Connection, ConnectionStatus>::new();

        let devices = Device::list().unwrap();
        let device = devices.into_iter()
            .find(|device| device.name == "wlo1".to_string())
            .unwrap();

        println!("device: {:?}", device);

        let mut cap = device.open().expect("Failed to load device");

        while let Ok(packet) = cap.next() {
            update_connections_with_inodes_from_receiver(&mut connections, &connections_thread);

            match process_packet(packet) {
                Err(_error) => {} //println!("Error: {}", error),
                Ok((connection, bytes_transferred)) => {
                    update_connections_with_bytes_transferred(&mut connections, connection, bytes_transferred);
                    updater.update(Some(connections.clone())).unwrap();
                }
            };
        }
    });

    (handle, receiver)
}

fn process_packet(packet: Packet) -> Result<(Connection, usize), std::string::String> {
    // Parse packet
    let packet_parse_result = SlicedPacket::from_ethernet(&packet);
    if let Err(error) = packet_parse_result {
        return Err(format!("Error in parsing packet: {:?}", error));
    }
    let packet_data = packet_parse_result.unwrap();


    // Get source and destination IPs
    if packet_data.ip == None {
        return Err("Received non-ip packet".to_string());
    }
    let ip = packet_data.ip.unwrap();
    let (source_ip, destination_ip) = match ip {
        Ipv4(header) => (header.source(), header.destination()),
        Ipv6(header, _) => (header.source(), header.destination())
    };


    // Get transport type
    if packet_data.transport == None {
        return Err("Received non-tcp/udp packet".to_string());
    }
    let transport = packet_data.transport.unwrap();
    let (transport_type, source_port, destination_port) = match transport {
        Tcp(header) => (TransportType::Tcp, header.source_port(), header.destination_port()),
        Udp(header) => (TransportType::Udp, header.source_port(), header.destination_port())
    };


    // Get packet size
    let packet_size = packet.len();

    let connection = Connection {
        source_ip: source_ip.to_vec(),
        source_port,
        destination_ip: destination_ip.to_vec(),
        destination_port,
        transport_type,
    };

    Ok((connection, packet_size))
}

fn update_connections_with_inodes_from_receiver(connections: &mut ConnectionWithStatus, receiver: &Receiver<HashMap<Connection, usize>>) {
    loop {
        match receiver.try_recv() {
            Ok(new_connections) => update_connections_with_inodes(connections, new_connections),
            Err(_) => break,
        }
    }
}

fn update_connections_with_inodes(connections: &mut ConnectionWithStatus, connection_inodes: HashMap<Connection, usize>) {
    for (connection, inode) in connection_inodes {
        if connections.contains_key(&connection) {
            let mut connection_status = connections.get_mut(&connection).unwrap();
            connection_status.inode = inode;
        } else {
            connections.insert(
                connection,
                ConnectionStatus { inode, bytes_transferred: 0 },
            );
        }
    }
}

fn update_connections_with_bytes_transferred(connections: &mut ConnectionWithStatus, connection: Connection, bytes_transferred: usize) {
    if connections.contains_key(&connection) {
        let mut connection_status = connections.get_mut(&connection).unwrap();
        connection_status.bytes_transferred += bytes_transferred;
    } else {
        if connection.transport_type == TransportType::Tcp {
            connections.insert(
                connection,
                ConnectionStatus { inode: 0, bytes_transferred: bytes_transferred },
            );
        }
//        for (connection, connection_status) in connections {
//            if connection.transport_type == TransportType::Tcp {
//                println!("{}", connection);
//            }
//        }
//        println!("Received packet from unknown connection: {:?}", connection);
    }
}
