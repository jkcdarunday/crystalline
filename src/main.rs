use std::env;
use std::sync::Mutex;

use actix_cors::Cors;
use actix_web::{App, rt, get, HttpResponse, HttpServer, middleware, web};
use serde_json::json;

use crate::structs::connection::Connections;
use crate::structs::receivers::{CaptureReceiver, ProcessesReceiver};
use crate::structs::process::ProcessInfos;

mod structs;
mod threads;
mod helpers;

#[get("/")]
async fn index(state: web::Data<Mutex<(CaptureReceiver, ProcessesReceiver, Connections, ProcessInfos)>>) -> HttpResponse {
    let mut locked_state = state.lock().unwrap();

    let (ref mut receiver, ref mut processes_receiver, ref mut connections, ref mut processes) = *locked_state;

    if let Some(latest_connections) = receiver.latest() {
        *connections = latest_connections.clone()
    }

    if let Some(latest_processes) = processes_receiver.latest() {
        processes.extend(latest_processes.clone());
    }

    for connection in connections.iter_mut() {
        connection.bind_matching_process(processes);
    }

    let filtered_processes: ProcessInfos = processes.iter()
        .filter(|&(_, process)| !process.executable.is_empty())
        .map(|(pid, process)| (pid.clone(), process.clone()))
        .collect();

    HttpResponse::Ok().json(json!({"connections": connections, "processes": filtered_processes}))
}

fn main() -> std::io::Result<()> {
    let device_name = env::args().nth(1);

    let (_, connections_thread) = threads::connections::run(200);
    let (_, processes_thread) = threads::processes::run(200);
    let (_, capture_thread) = threads::capture::run(connections_thread, &device_name);


    let state = web::Data::new(Mutex::new((capture_thread, processes_thread, Connections::new(), ProcessInfos::new())));

    let host = env::var("HOST").unwrap_or("127.0.0.1:8080".to_string());
    println!("Starting server at {}...", host);
    rt::System::new().block_on(HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(middleware::Logger::default())
            .wrap(Cors::permissive().allowed_methods(vec!["GET"]).max_age(3600))
            .service(index)
    })
        .bind(host)?
        .run()
    )
}
