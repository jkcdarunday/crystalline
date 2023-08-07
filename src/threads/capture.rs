use std::mem::swap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

use etherparse::InternetSlice::Ipv4;
use etherparse::InternetSlice::Ipv6;
use etherparse::SlicedPacket;
use etherparse::TransportSlice::{Tcp, Udp};
use pcap::{Device, Packet};
use single_value_channel;
use crate::helpers::display::print_devices;

use crate::structs::connection::{Connection, Connections, TransportType};
use crate::structs::receivers::{CaptureReceiver, ConnectionsReceiver};

pub type CaptureUpdater = single_value_channel::Updater<Option<Connections>>;

pub fn run(connections_thread: ConnectionsReceiver, device_name: &Option<String>) -> (Vec<JoinHandle<()>>, CaptureReceiver) {
    let (receiver, updater) = single_value_channel::channel();

    let devices = Device::list().unwrap().into_iter().filter(|device|{
        !device.flags.is_loopback()
            && device.flags.is_up()
            && device.flags.is_running()
            && device.name != "any"
            && (device_name.is_none() || device.name == *device_name.as_ref().unwrap())
    }).collect();
    print_devices(&devices);

    let connections_mutex = Arc::new(Mutex::new(Connections::new()));
    let receiver_mutex = Arc::new(Mutex::new(connections_thread));
    let updater = Arc::new(Mutex::new(updater));
    let handles: Vec<JoinHandle<()>> = devices.into_iter()
        .map(|device| {
        let connections_mutex_instance = connections_mutex.clone();
        let receiver_mutex_instance = receiver_mutex.clone();
        let updater = updater.clone();

        thread::spawn(move ||
            monitor_device(device, &connections_mutex_instance, &receiver_mutex_instance, &updater)
        )
    }).collect();

    println!("Started {} capture threads", handles.len());

    (handles, receiver)
}

fn monitor_device(device: Device, connections_mutex: &Mutex<Connections>, receiver_mutex: &Mutex<ConnectionsReceiver>, updater_mutex: &Mutex<CaptureUpdater>) {
    let mut cap = device.open().expect("Failed to load device");

    while let Ok(packet) = cap.next_packet() {
        {
            let mut connections = connections_mutex.lock().unwrap();
            let mut receiver = receiver_mutex.lock().unwrap();
            update_connections_with_inodes_from_receiver(&mut connections, &mut receiver);
        }

        match process_packet(packet) {
            Err(error) => println!("Error: {}", error),
            Ok((connection, bytes_transferred)) => {
                let mut connections = connections_mutex.lock().unwrap();
                let updater = updater_mutex.lock().unwrap();
                update_connections_with_bytes_transferred(&mut connections, connection, bytes_transferred);
                updater.update(Some(connections.clone())).unwrap();
            }
        };
    }
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
        Ipv4(header, _) => (IpAddr::V4(header.source_addr()), IpAddr::V4(header.destination_addr())),
        Ipv6(header, _) => (IpAddr::V6(header.source_addr()), IpAddr::V6(header.destination_addr()))
    };

    let connection = match packet_data.transport.unwrap() {
        Tcp(header) => Connection {
            source: SocketAddr::new(source_ip, header.source_port()),
            destination: SocketAddr::new(destination_ip, header.destination_port()),
            inode: 0,
            process_id: 0,
            transport_type: TransportType::Tcp,
            bytes_uploaded: 0,
            bytes_downloaded: 0,
        },
        Udp(header) => Connection {
            source: SocketAddr::new(source_ip, header.source_port()),
            destination: SocketAddr::new(destination_ip, header.destination_port()),
            inode: 0,
            process_id: 0,
            transport_type: TransportType::Udp,
            bytes_uploaded: 0,
            bytes_downloaded: 0,
        },
        _ => return Err("Received non-tcp/udp packet".to_string())
    };

    let packet_size = packet.len();

    Ok((connection, packet_size))
}

fn update_connections_with_inodes_from_receiver(connections: &mut Connections, receiver: &mut ConnectionsReceiver) {
    let new_connections = match receiver.latest() {
        Some(connections) => connections,
        None => return
    };

    for new_connection in new_connections {
        let find_connection_result = connections.iter_mut().find(|current| **current == *new_connection);
        if let Some(found_connection) = find_connection_result {
            if found_connection.inode == 0 {
                found_connection.inode = new_connection.inode;
            }

            if found_connection.source == new_connection.destination {
                swap(&mut found_connection.source, &mut found_connection.destination);
                swap(&mut found_connection.bytes_uploaded, &mut found_connection.bytes_downloaded);
            }
        } else {
            connections.push(new_connection.clone());
        }
    }
}

fn update_connections_with_bytes_transferred(connections: &mut Connections, connection: Connection, bytes_transferred: usize) {
    if let Some(found_connection) = connections.iter_mut().find(|current| **current == connection) {
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

    connections.sort_unstable_by(|a, b| b.cmp(a));
}
