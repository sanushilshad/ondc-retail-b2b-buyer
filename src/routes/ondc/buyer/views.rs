use actix::Addr;
use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;

use crate::websocket::{MessageToClient, Server};

use super::{schemas::ONDCOnSearchRequest, utils::fetch_websocket_params};

#[tracing::instrument(name = "ONDC On Search Response", skip(pool), fields())]
pub async fn on_search(
    pool: web::Data<PgPool>,
    // req_body: web::Bytes,
    body: web::Json<ONDCOnSearchRequest>,
    websocket_srv: web::Data<Addr<Server>>,
) -> impl Responder {
    let json_obj = serde_json::to_value(&body.0).unwrap();
    match fetch_websocket_params(
        &pool,
        &body.context.transaction_id,
        &body.context.message_id,
    )
    .await
    {
        Ok(Some(ws_params)) => {
            println!("{}", ws_params.get_key());
            let msg = MessageToClient::new("search", json_obj.clone(), Some(ws_params.get_key()));
            websocket_srv.do_send(msg);
        }
        Ok(None) => {}
        Err(_) => {
            return HttpResponse::InternalServerError().body("Error fetching WebSocket parameters")
        }
    };

    HttpResponse::Ok().body(" BLAH Running")
}
