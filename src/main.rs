use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::Mutex;

use actix_cors::Cors;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, middleware, web};
use serde_json::json;

use crate::structs::connection::{Connection, ConnectionStatus};

mod structs;
mod threads;

fn index(state: web::Data<Mutex<(single_value_channel::Receiver<Option<HashMap<Connection, ConnectionStatus>>>, HashMap<Connection, ConnectionStatus>)>>, req: HttpRequest) -> HttpResponse {
    println!("{:?}", req);
    let mut locked_state = state.lock().unwrap();

    let (ref mut receiver, ref mut connections) = *locked_state;

    if let Some(latest_connections) = receiver.latest() {
        *connections = latest_connections.clone()
    }

    let mut retval = vec![];

    for (connection, status) in connections {
        retval.push(json!({"connection": connection, "status": status}));
    };

    HttpResponse::Ok().json(retval)
}

fn main() -> std::io::Result<()> {
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
