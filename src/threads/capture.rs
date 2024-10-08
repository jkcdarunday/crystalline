use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::{thread};
use std::thread::JoinHandle;
use std::time::SystemTime;

use etherparse::InternetSlice::Ipv4;
use etherparse::InternetSlice::Ipv6;
use etherparse::SlicedPacket;
use etherparse::TransportSlice::{Tcp, Udp};
use pcap::{Device, Linktype, Packet};
use single_value_channel;
use crate::helpers::debug::is_debug;
use crate::helpers::display::print_devices;

use crate::structs::connection::{Connection, Connections, TransportType};
use crate::structs::receivers::{CaptureReceiver, ConnectionsReceiver};

pub type CaptureUpdater = single_value_channel::Updater<Option<Connections>>;

pub enum Direction {
    Incoming,
    Outgoing,
}

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
    let updater_mutex = Arc::new(Mutex::new(updater));
    let handles: Vec<JoinHandle<()>> = devices.into_iter()
        .map(|device| {
        let connections_mutex_instance = connections_mutex.clone();
        let receiver_mutex_instance = receiver_mutex.clone();
        let updater = updater_mutex.clone();

        thread::spawn(move ||
            monitor_device(device, &connections_mutex_instance, &receiver_mutex_instance, &updater)
        )
    }).collect();

    println!("Started {} capture threads", handles.len());

    (handles, receiver)
}

fn monitor_device(device: Device, connections_mutex: &Mutex<Connections>, receiver_mutex: &Mutex<ConnectionsReceiver>, updater_mutex: &Mutex<CaptureUpdater>) {
    let addresses = device.addresses.iter().map(|address| address.addr).collect::<Vec<IpAddr>>();
    let mut cap = device.open().expect("Failed to load device");
    let link_type = cap.get_datalink();

    while let Ok(packet) = cap.next_packet() {
        {
            let mut connections = connections_mutex.lock().unwrap();
            let mut receiver = receiver_mutex.lock().unwrap();
            update_connections_with_inodes_from_receiver(&mut connections, &mut receiver);
        }

        match process_packet(packet, link_type, &addresses) {
            Err(error) => if is_debug() { println!("Error: {}", error) },
            Ok((connection, bytes_transferred, direction)) => {
                let mut connections = connections_mutex.lock().unwrap();
                let updater = updater_mutex.lock().unwrap();

                update_connections_with_bytes_transferred(&mut connections, connection, bytes_transferred, direction);
                updater.update(Some(connections.clone())).unwrap();
            }
        };
    }
}

fn process_packet(packet: Packet, link_type: Linktype, addresses: &Vec<IpAddr>) -> Result<(Connection, usize, Direction), String> {
    // Parse packet
    let packet_parse_result = match link_type {
        Linktype(12) | Linktype::NULL => SlicedPacket::from_ip(&packet),
        Linktype::ETHERNET => SlicedPacket::from_ethernet(&packet),
        _ => return Err(format!("Unsupported link type {:?}", link_type.get_description()))
    };
    if let Err(error) = packet_parse_result {
        return Err(format!("Error in parsing packet: {:?}", error));
    }
    let packet_data = packet_parse_result.unwrap();


    // Get source and destination IPs
    if packet_data.net.is_none() {
        return Err("Received non-ip packet".to_string());
    }

    // Get transport type
    if packet_data.transport.is_none() {
        return Err("Received non-tcp/udp packet".to_string());
    }

    let (source_ip, destination_ip) = match packet_data.net.unwrap() {
        Ipv4(slice) => (IpAddr::V4(slice.header().source_addr()), IpAddr::V4(slice.header().destination_addr())),
        Ipv6(slice) => (IpAddr::V6(slice.header().source_addr()), IpAddr::V6(slice.header().destination_addr()))
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
            first_seen: SystemTime::now(),
            last_seen: SystemTime::now(),
        },
        Udp(header) => Connection {
            source: SocketAddr::new(source_ip, header.source_port()),
            destination: SocketAddr::new(destination_ip, header.destination_port()),
            inode: 0,
            process_id: 0,
            transport_type: TransportType::Udp,
            bytes_uploaded: 0,
            bytes_downloaded: 0,
            first_seen: SystemTime::now(),
            last_seen: SystemTime::now(),
        },
        _ => return Err("Received non-tcp/udp packet".to_string())
    };

    let packet_size = packet.len();

    let direction = match addresses {
        _ if addresses.contains(&source_ip) => Direction::Outgoing,
        _ if addresses.contains(&destination_ip) => Direction::Incoming,
        _ => return Err("Packet not from or to monitored device".to_string())
    };

    Ok((connection, packet_size, direction))
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
        } else {
            connections.push(new_connection.clone());
        }
    }
}

fn update_connections_with_bytes_transferred(connections: &mut Connections, connection: Connection, bytes_transferred: usize, direction: Direction) {
    if let Some(found_connection) = connections.iter_mut().find(|current| **current == connection) {
        match direction {
            Direction::Outgoing => found_connection.bytes_uploaded += bytes_transferred,
            Direction::Incoming => found_connection.bytes_downloaded += bytes_transferred,
        }
    } else {
        let mut new_connection = connection.clone();
        match direction {
            Direction::Outgoing => new_connection.bytes_uploaded += bytes_transferred,
            Direction::Incoming => new_connection.bytes_downloaded += bytes_transferred,
        }
        connections.push(new_connection);
    }

    connections.sort_unstable_by(|a, b| b.cmp(a));
}
