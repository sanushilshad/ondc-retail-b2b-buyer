use crate::middleware::RequireAuth;

use actix_web::web;

use super::handlers::send_email_otp;
pub fn notification_route(cfg: &mut web::ServiceConfig) {
    // cfg.service(web::resource("/send/email").route(web::post().to(send_email)));
    cfg.service(
        web::resource("/send/email/otp").route(web::post().to(send_email_otp).wrap(RequireAuth)),
    );
    // cfg.route("/customer/database", web::post().to(get_customer_dbs_api))
}
