use actix_web::{web, Responder};
use sqlx::PgPool;

#[tracing::instrument(name = "Authenticate User", skip(_pool), fields())]
pub async fn authenticate(_pool: web::Data<PgPool>) -> impl Responder {
    let _request_span = tracing::info_span!("starting fetching of databases.");
    // tracing::info!("request_id {} - fetching databases.", request_id);

    web::Json({})
}
