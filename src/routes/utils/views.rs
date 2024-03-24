use crate::websocket::MyWs;

// use crate::routes::utils::get_customer_dbs;
use super::utils::get_customer_dbs;
use actix_web::{web, Error, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
use sqlx::PgPool;

pub async fn health_check() -> impl Responder {
    println!("mango");
    HttpResponse::Ok().body("Running")
}

#[tracing::instrument(
    name = "Fetching Customer List",
    skip(pool),
    fields(
    // request_id = %Uuid::new_v4()
    )
    )]
pub async fn get_customer_dbs_api(pool: web::Data<PgPool>) -> impl Responder {
    let _request_span = tracing::info_span!("starting fetching of databases.");
    // tracing::info!("request_id {} - fetching databases.", request_id);
    let db_domain_mapping = get_customer_dbs(pool)
        .await
        .expect("Something went wrong with the db connection");
    // match get_customer_dbs().await {
    //     Ok(data) => {
    //         log::info!("Sucessfully fetched data");
    //         web::Json(db_domain_mapping)
    //     }
    //     Err(err) => {
    //         log::error!("Operation failed: {}", err)
    //     }
    // }

    web::Json(db_domain_mapping)
}

#[tracing::instrument(
    name = "web_socket",
    skip(stream),
    fields(
    // request_id = %Uuid::new_v4()
    )
    )]
pub async fn web_socket(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp: Result<HttpResponse, Error> = ws::start(MyWs {}, &req, stream);
    println!("SANU {:?}", resp);
    resp
}
