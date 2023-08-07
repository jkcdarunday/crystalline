use pcap::{Address, Device};

pub fn format_addresses(addresses: &Vec<Address>) -> String {
    let mut result = String::new();

    for address in addresses {
        result.push_str(&format!("{} ", address.addr));
    }

    result
}

pub fn print_devices(devices: &Vec<Device>) {
    println!("Devices:");
    for device in devices {
        println!("\t- {}: {}", device.name, format_addresses(&device.addresses));
    }
}
