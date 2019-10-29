mod structs;
mod threads;

fn main() {
    let (_, connections_thread) = threads::connections::run(1000);
    let (_, processes_thread) = threads::processes::run(1000);
    let (capture_handle, capture_thread) = threads::capture::run(connections_thread, processes_thread);

    capture_handle.join();
}
