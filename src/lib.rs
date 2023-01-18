//! src/lib.rs
use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer};
use std::net::TcpListener;
// use serde::Serialize;
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().body("Hello Sanu")
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| App::new().route("/health_check", web::get().to(health_check)))
        .listen(listener)?
        .run();

    Ok(server)
}
