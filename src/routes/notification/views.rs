use std::{collections::HashMap, sync::Arc};

use actix_web::web;

use crate::routes::user::schemas::UserAccount;
use crate::{
    email_client::GenericEmailService,
    schemas::{CommunicationType, GenericResponse},
};

use super::{errors::OTPError, schemas::OTPRequestBody, utils::send_email_background};

// #[tracing::instrument(name = "Sending Email", skip(email_client), fields())]
// pub async fn send_email(email_client: web::Data<EmailClient>) -> impl Responder {
//     let _responsed = email_client
//         .send_text_email("sanu.shilshad@acelrtech.com", "SANU", "apple".to_owned())
//         .await;

//     HttpResponse::Ok().body("Successfully send data")
// }

#[tracing::instrument(
    name = "Sending OTP",
    skip(email_client, req_body),
    fields(email_client)
)]
pub async fn send_email_otp(
    email_client: web::Data<HashMap<CommunicationType, Arc<dyn GenericEmailService>>>,
    req_body: web::Json<OTPRequestBody>,
    user: UserAccount,
) -> Result<web::Json<GenericResponse<()>>, OTPError> {
    if let Some(email_service) = email_client.get(&CommunicationType::Type1) {
        tokio::spawn(send_email_background(
            email_service.clone(),
            req_body.identifier.get().to_string(),
        ));
    } else {
        return Err(OTPError::UnexpectedStringError(
            "Internal Server Error".to_string(),
        ));
    }

    Ok(web::Json(GenericResponse::success(
        "Successfully Send OTP",
        Some(()),
    )))
}
