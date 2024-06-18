use actix_web::{HttpResponse, Responder};

pub async fn ondc_seller_sample() -> impl Responder {
    println!("mango");
    HttpResponse::Ok().body("Running")
}
