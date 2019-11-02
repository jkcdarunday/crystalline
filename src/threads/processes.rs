use std::collections::HashMap;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use libc::pid_t;
use procfs::process::FDTarget::{Net, Other, Pipe, Socket};

pub fn run(
    interval: u64,
) -> (
    JoinHandle<()>,
    single_value_channel::Receiver<Option<HashMap<pid_t, Vec<u32>>>>,
) {
    let (receiver, updater) = single_value_channel::channel();

    let handle = thread::spawn(move || loop {
        updater.update(Some(get_inodes_per_process())).unwrap();
        thread::sleep(Duration::from_millis(interval));
    });

    (handle, receiver)
}

pub fn get_inodes_per_process() -> HashMap<pid_t, Vec<u32>> {
    let mut process_inodes = HashMap::new();

    for process in procfs::process::all_processes().unwrap() {
        let mut inodes = Vec::new();

        if let Ok(file_descriptors) = process.fd() {
            for file_descriptor in file_descriptors {
                match file_descriptor.target {
                    Socket(inode) | Net(inode) | Pipe(inode) | Other(_, inode) => inodes.push(inode),
                    _ => {}
                }
            }
        }

        process_inodes.insert(process.pid(), inodes);
    }

    process_inodes
}
