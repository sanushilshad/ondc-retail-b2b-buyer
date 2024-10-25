use utoipa::OpenApi;
use utoipauto::utoipauto;
#[utoipauto]
#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "ONDC Buyer REST API", description = "ONDC Buyer App API Endpoints")
    ),
    info(
        title = "ONDC Buyer API",
        description = "ONDC Buyer API Endpoints",
        version = "1.0.0",
        license(name = "MIT", url = "https://opensource.org/licenses/MIT")
    ),
)]

pub struct ApiDoc {}
