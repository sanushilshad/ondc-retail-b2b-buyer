use crate::domain::EmailObject;
use crate::routes::user::schemas::{
    CreateBusinessAccount, CreateUserAccount, CustomerType, DataSource, KYCProof, MerchantType,
    TradeType, UserType, VectorType, AuthenticateRequest, AuthenticationScope, AuthData, UserAccount, BasicBusinessAccount, UserVector, MaskingType 
};
use crate::routes::user::views as user_views;
use crate::schemas::{EmptyGenericResponse, AuthResponse, Status};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        user_views::register_user_account,
        user_views::register_business_account,
        user_views::authenticate

    ),
    components(
        schemas(CreateUserAccount, UserType, DataSource, EmailObject, EmptyGenericResponse, CreateBusinessAccount, CustomerType, MerchantType, KYCProof, TradeType, VectorType, 
            AuthenticateRequest, AuthResponse, AuthData, AuthenticationScope, UserAccount, BasicBusinessAccount, UserVector, MaskingType, Status )
    ),
    tags(
        (name = "Rust REST API", description = "Authentication in Rust Endpoints")
    ),
)]

pub struct ApiDoc;
