use crate::domain::EmailObject;
use crate::routes::user::schemas::{
    CreateBusinessAccount, CreateUserAccount, CustomerType, DataSource, KYCProof, MerchantType,
    TradeType, UserType, VectorType, AuthenticateRequest, AuthenticationScope, AuthData, UserAccount, BasicBusinessAccount, UserVector, MaskingType 
};
use crate::routes::user::handlers as user_handlers;
use crate::schemas::{AuthResponse, CountryCode, EmptyGenericResponse, Status};
use utoipa::OpenApi;
use crate::routes::product::handlers as product_handlers;
use crate::routes::product::schemas::{ProductSearchRequest, PaymentType, ProductSearchType, CategoryDomain, ProductFulFillmentLocations, FulfillmentType};
#[derive(OpenApi)]
#[openapi(
    paths(
        user_handlers::register_user_account,
        user_handlers::register_business_account,
        user_handlers::authenticate,
        product_handlers::realtime_product_search


    ),
    components(
        schemas(CreateUserAccount, UserType, DataSource, EmailObject, EmptyGenericResponse, CreateBusinessAccount, CustomerType, MerchantType, KYCProof, TradeType, VectorType, 
            AuthenticateRequest, AuthResponse, AuthData, AuthenticationScope, UserAccount, BasicBusinessAccount, UserVector, MaskingType, Status, ProductSearchRequest, PaymentType, ProductSearchType,
            CategoryDomain, CountryCode, ProductFulFillmentLocations, FulfillmentType )
    ),
    tags(
        (name = "Rust REST API", description = "Authentication in Rust Endpoints")
    ),
)]

pub struct ApiDoc;
