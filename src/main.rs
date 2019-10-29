use std::thread;
use std::time::Duration;
use std::collections::HashMap;

mod structs;
mod threads;

fn main() {
    let (_, connections_thread) = threads::connections::run(1000);
    let (_, processes_thread) = threads::processes::run(1000);
    let (_, capture_thread) = threads::capture::run(connections_thread, processes_thread);

    let mut connections= HashMap::new();
    loop {
        print!("Updating connections... ");
        let mut latest_connections = capture_thread.try_recv();
        while latest_connections.is_ok() {
            connections = latest_connections.unwrap();
            latest_connections = capture_thread.try_recv()
        }

        println!("OK");

        if connections.len() > 0 {
            print!("{}[2J", 27 as char);
            for (connection, connection_status) in &connections {
                println!("{} | {} bytes", connection, connection_status.bytes_transferred);
            }
        }

        thread::sleep(Duration::from_millis(1000));
    }
}
