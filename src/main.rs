use std::sync::Mutex;

use actix_cors::Cors;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, middleware, web};
use serde_json::json;

use crate::structs::connection::Connections;
use crate::structs::receivers::{CaptureReceiver, ProcessesReceiver};
use crate::structs::process::ProcessInfos;

mod structs;
mod threads;

fn index(state: web::Data<Mutex<(CaptureReceiver, ProcessesReceiver, Connections, ProcessInfos)>>, _req: HttpRequest) -> HttpResponse {
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

    HttpResponse::Ok().json(json!({"connections": connections, "processes": processes}))
}

fn main() -> std::io::Result<()> {
    let (_, connections_thread) = threads::connections::run(200);
    let (_, processes_thread) = threads::processes::run(200);
    let (_, capture_thread) = threads::capture::run(connections_thread);


    let state = web::Data::new(Mutex::new((capture_thread, processes_thread, Connections::new(), ProcessInfos::new())));

    HttpServer::new(move || App::new()
        .register_data(state.clone())
        .wrap(middleware::Logger::default())
        .wrap(Cors::new().allowed_methods(vec!["GET"]).max_age(3600))
        .service(web::resource("/").to(index))
    )
        .bind("127.0.0.1:8080")?
        .run()
}
