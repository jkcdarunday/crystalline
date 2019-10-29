use std::collections::HashMap;
use std::{fs, thread};
use std::os::unix::fs::MetadataExt;
use std::sync::mpsc::{Receiver, channel};
use std::thread::JoinHandle;
use std::time::Duration;

pub fn run(interval: u64) -> (JoinHandle<()>, Receiver<HashMap<usize, Vec<u64>>>) {
    let (sender, receiver) = channel();

    let handle = thread::spawn(move || {
        loop {
            sender.send(get_inodes_per_process()).unwrap();
            thread::sleep(Duration::from_millis(interval));
        }
    });

    (handle, receiver)
}


pub fn get_inodes_per_process() -> HashMap<usize, Vec<u64>> {
    let mut process_inodes  = HashMap::new();
    let proc_dirs = fs::read_dir("/proc").unwrap();

    for proc_dir_result in proc_dirs {
        let proc_dir = proc_dir_result.unwrap();
        let proc_dir_filename= proc_dir.file_name().into_string().unwrap();

        // Check if filename is parseable as number
        let process_id = match proc_dir_filename.parse::<usize>() {
            Ok(process_id) => process_id,
            Err(_) => continue
        };

        let process_fd_paths = match fs::read_dir(format!("/proc/{}/fd", process_id)) {
            Ok(process_fd) => process_fd,
            Err(_) => continue
        };

        let mut inodes = Vec::new();
        for process_fd_path in process_fd_paths {
            let fd = process_fd_path.unwrap();
            let inode = match fd.metadata() {
                Ok(metadata) => metadata.ino(),
                Err(_) => continue
            };

            inodes.push(inode);
        }

        process_inodes.insert(process_id, inodes);
    }

    process_inodes
}
