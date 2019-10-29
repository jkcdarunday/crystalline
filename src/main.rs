use std::collections::HashMap;
use std::sync::Mutex;
use crate::structs::connection::{ConnectionStatus, Connection};
use actix_web::{web, App, HttpResponse, HttpRequest, HttpServer, middleware};
use std::sync::mpsc::Receiver;
use serde_json::json;
use actix_cors::Cors;

mod structs;
mod threads;

fn index(state: web::Data<Mutex<(Receiver<HashMap<Connection, ConnectionStatus>>, HashMap<Connection, ConnectionStatus>)>>, req: HttpRequest) -> HttpResponse {
    println!("{:?}", req);
    let mut locked_state = state.lock().unwrap();

    let mut latest_connections = locked_state.0.try_recv();
    while latest_connections.is_ok() {
        locked_state.1 = latest_connections.unwrap();
        latest_connections = locked_state.0.try_recv()
    }

    let mut retval = vec![];

    for (connection, status) in &locked_state.1 {
        retval.push(json!({"connection": connection, "status": status}));
    };

    HttpResponse::Ok().json(retval)
}

fn main() -> std::io::Result<()>{
    let (_, connections_thread) = threads::connections::run(1000);
    let (_, processes_thread) = threads::processes::run(1000);
    let (_, capture_thread) = threads::capture::run(connections_thread, processes_thread);


    let state = web::Data::new(Mutex::new((capture_thread, HashMap::<Connection, ConnectionStatus>::new())));

    HttpServer::new(move || App::new()
        .register_data(state.clone())
        .wrap(middleware::Logger::default())
        .wrap(Cors::new().allowed_methods(vec!["GET"]).max_age(3600))
        .service(web::resource("/").to(index))
    )
        .bind("127.0.0.1:8080")?
        .run()
}
