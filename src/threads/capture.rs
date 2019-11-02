use std::mem::swap;
use std::net::{IpAddr, SocketAddr};
use std::thread;
use std::thread::JoinHandle;

use etherparse::InternetSlice::Ipv4;
use etherparse::InternetSlice::Ipv6;
use etherparse::SlicedPacket;
use etherparse::TransportSlice::{Tcp, Udp};
use pcap::{Device, Packet};
use single_value_channel;

use crate::structs::connection::{Connection, Connections, TransportType};
use crate::structs::receivers::{CaptureReceiver, ConnectionsReceiver};

pub fn run(connections_thread: ConnectionsReceiver) -> (JoinHandle<()>, CaptureReceiver) {
//    let (sender, receiver) = channel();
    let (receiver, updater) = single_value_channel::channel();

    let handle = thread::spawn(move || {
        let mut connections = Connections::new();

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

    // Get transport type
    if packet_data.transport == None {
        return Err("Received non-tcp/udp packet".to_string());
    }

    let (source_ip, destination_ip) = match packet_data.ip.unwrap() {
        Ipv4(header) => (IpAddr::V4(header.source_addr()), IpAddr::V4(header.destination_addr())),
        Ipv6(header, _) => (IpAddr::V6(header.source_addr()), IpAddr::V6(header.destination_addr()))
    };

    let connection = match packet_data.transport.unwrap() {
        Tcp(header) => Connection {
            source: SocketAddr::new(source_ip, header.source_port()),
            destination: SocketAddr::new(destination_ip, header.destination_port()),
            inode: 0,
            transport_type: TransportType::Tcp,
            bytes_uploaded: 0,
            bytes_downloaded: 0,
        },
        Udp(header) => Connection {
            source: SocketAddr::new(source_ip, header.source_port()),
            destination: SocketAddr::new(destination_ip, header.destination_port()),
            inode: 0,
            transport_type: TransportType::Udp,
            bytes_uploaded: 0,
            bytes_downloaded: 0,
        }
    };

    let packet_size = packet.len();

    Ok((connection, packet_size))
}

fn update_connections_with_inodes_from_receiver(connections: &mut Connections, receiver: &ConnectionsReceiver) {
    while let Ok(new_connections) = receiver.try_recv() {
        for new_connection in new_connections {
            if let Some(mut found_connection) = connections.iter_mut().find(|current| **current == new_connection) {
                if found_connection.inode == 0 {
                    found_connection.inode = new_connection.inode;
                }

                if found_connection.source == new_connection.destination {
                    swap(&mut found_connection.source, &mut found_connection.destination);
                    swap(&mut found_connection.bytes_uploaded, &mut found_connection.bytes_downloaded);
                }
            } else {
                connections.push(new_connection);
            }
        }
    }
}

fn update_connections_with_bytes_transferred(connections: &mut Connections, connection: Connection, bytes_transferred: usize) {
    if let Some(mut found_connection) = connections.iter_mut().find(|current| **current == connection) {
        if found_connection.source == connection.source {
            found_connection.bytes_uploaded += bytes_transferred;
        } else {
            found_connection.bytes_downloaded += bytes_transferred;
        }
    } else {
        let mut new_connection = connection.clone();
        new_connection.bytes_uploaded = bytes_transferred;
        connections.push(new_connection);
    }
}
