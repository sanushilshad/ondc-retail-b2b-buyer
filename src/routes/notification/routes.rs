use super::views::send_email;
use actix_web::web;
pub fn notification_route(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/send/email").route(web::post().to(send_email)));
    // cfg.route("/customer/database", web::post().to(get_customer_dbs_api))
}
