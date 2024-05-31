use actix_web::{HttpResponse, Responder};

pub async fn on_search() -> impl Responder {
    println!("mango");
    HttpResponse::Ok().body("Running")
}
