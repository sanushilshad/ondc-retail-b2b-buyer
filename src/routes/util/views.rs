// use crate::{redis::RedisClient, websocket::MyWs};
use crate::{
    redis::RedisClient,
    schemas::WebSocketParam,
    websocket::{Server, WebSocketSession},
};
// use crate::routes::utils::get_customer_dbs;
use super::{
    schemas::{RedisAction, RedisBasicRequest},
    utils::get_customer_dbs,
};
use actix::Addr;
use actix_web::{web, Error, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
use redis::AsyncCommands;
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
    name = "Commect web socket",
    skip(stream),
    fields(
    // request_id = %Uuid::new_v4()
    )
    )]
pub async fn web_socket(
    req: HttpRequest,
    stream: web::Payload,
    query: web::Query<WebSocketParam>,
    server_addr: web::Data<Addr<Server>>,
) -> Result<HttpResponse, Error> {
    let web_socket_key = query.get_key();
    println!("{}", web_socket_key);
    let res = ws::start(
        WebSocketSession::new(web_socket_key, server_addr.get_ref().clone()),
        &req,
        stream,
    )?;
    Ok(res)
}

#[tracing::instrument(name = "Set Value in Redis", skip(redis), fields())]
pub async fn redis_basic(
    redis: web::Data<RedisClient>,
    body: web::Json<RedisBasicRequest>,
) -> HttpResponse {
    let mut conn = redis
        .get_connection()
        .await
        .expect("Failed to get Redis connection");
    let message: String;
    match body.action {
        RedisAction::Set => {
            let _: () = conn
                .set(&body.key, &body.value)
                .await
                .expect("Failed to set value in Redis");
            message = "Successfully set value in Redis".to_string()
        }
        RedisAction::Get => {
            let value: Option<String> = conn
                .get(&body.key)
                .await
                .expect("Failed to get value from Redis");
            message = value.unwrap_or_default();
        }
    }

    HttpResponse::Ok().body(message)
}
