use secrecy::Secret;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

// macro_rules! impl_serialize_format {
//     ($struct_name:ident, $trait_name:path) => {
//         impl $trait_name for $struct_name {
//             fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//                 fmt_json(self, f)
//             }
//         }
//     };
// }

// #[derive(Debug)]
// struct SecretString(Secret<String>);

// impl_serialize_format!(AuthenticateRequest, Debug);
// #[strum(serialize_all = "snake_case")]
// #[derive(Debug, Deserialize, Serialize)]
// #[serde(rename_all = "lowercase")]

// pub enum AuthenticationScope {
//     OTP,
//     Password,
// }

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateRequest {
    pub scope: AuthenticationScope,
    pub identifier: String,
    // #[serde(with = "SecretString")]
    pub secret: Secret<String>,
}

// impl Serialize for SecretString {
//     fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
//         serializer.serialize_str(&self.0.expose_secret())
//     }
// }

// pub struct AuthData {
//     token: String,
// }

// pub struct AuthResponse {
//     data: AuthData,
// }

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
#[sqlx(type_name = "user_auth_identifier_scope", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum AuthenticationScope {
    Otp,
    Password,
    Google,
    Facebook,
    Microsoft,
    Apple,
    Token,
    AuthApp,
    Qr,
    Email,
}
