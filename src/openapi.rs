use crate::routes::user::schemas::AuthData;
use utoipa::openapi::Object;
use utoipa::OpenApi;
use utoipauto::utoipauto;
#[utoipauto]
#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "ONDC Buyer REST API", description = "ONDC Buyer App API Endpoints")
    ),
)]

pub struct ApiDoc {}
