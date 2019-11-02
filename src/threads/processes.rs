use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use procfs::process::FDTarget::{Net, Other, Pipe, Socket};

use crate::structs::process::{ProcessInfo, ProcessInfos};

pub fn run(interval: u64) -> (JoinHandle<()>, single_value_channel::Receiver<Option<ProcessInfos>>) {
    let (receiver, updater) = single_value_channel::channel();

    let handle = thread::spawn(move || loop {
        updater.update(Some(get_inodes_per_process())).unwrap();
        thread::sleep(Duration::from_millis(interval));
    });

    (handle, receiver)
}

pub fn get_inodes_per_process() -> ProcessInfos {
    let mut process_infos = ProcessInfos::new();

    for process in procfs::process::all_processes().unwrap() {
        let mut process_info = ProcessInfo {
            pid: process.pid(),
            command: process.cmdline().unwrap().join(" "),
            executable: String::from(process.exe().unwrap_or_default().to_str().unwrap_or("")),
            inodes: Vec::new()
        };

        if let Ok(file_descriptors) = process.fd() {
            for file_descriptor in file_descriptors {
                match file_descriptor.target {
                    Socket(inode) | Net(inode) | Pipe(inode) | Other(_, inode) => {
                        process_info.inodes.push(inode)
                    }
                    _ => {}
                }
            }
        }

        process_infos.push(process_info);
    }

    process_infos
}
