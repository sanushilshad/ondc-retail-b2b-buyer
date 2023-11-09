use actix_web::{web, HttpResponse, Responder};

use crate::email_client::EmailClient;

#[tracing::instrument(name = "Sending Email", skip(email_client), fields())]
pub async fn send_email(email_client: web::Data<EmailClient>) -> impl Responder {
    let _responsed = email_client
        .send_email_smtp("sanu.shilshad@acelrtech.com", "SANU", "apple".to_owned())
        .await;

    HttpResponse::Ok().body("Successfully send data")
}
