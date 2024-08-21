use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    time::Duration,
};

use crate::errors::RequestMetaError;
use crate::routes::user::schemas::AuthData;
use actix_web::{error::ErrorInternalServerError, FromRequest, HttpMessage};
use bigdecimal::BigDecimal;
use futures_util::future::{ready, Ready};
use reqwest::{header, Client};
use secrecy::Secret;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::postgres::PgHasArrayType;
use tokio::time::sleep;
use utoipa::{openapi::Object, ToSchema};
use uuid::Uuid;

#[derive(Serialize, Debug, ToSchema)]
#[aliases(EmptyGenericResponse = GenericResponse<Object>, AuthResponse = GenericResponse<AuthData>)]
pub struct GenericResponse<D> {
    pub status: bool,
    pub customer_message: String,
    pub code: String,
    pub data: Option<D>,
}

impl<D> GenericResponse<D> {
    pub fn success(message: &str, data: Option<D>) -> Self {
        Self {
            status: true,
            customer_message: String::from(message),
            code: String::from("200"),
            data,
        }
    }

    pub fn error(message: &str, code: &str, data: Option<D>) -> Self {
        Self {
            status: false,
            customer_message: String::from(message),
            code: String::from(code),
            data,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, Clone, PartialEq, ToSchema)]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "status")]
pub enum Status {
    Active,
    Inactive,
    Pending,
    Archived,
}

impl PgHasArrayType for Status {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_status")
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommunicationType {
    Type1,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JWTClaims {
    pub sub: Uuid,
    pub exp: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum CountryCode {
    AFG, // Afghanistan
    ALA, // Åland Islands
    ALB, // Albania
    DZA, // Algeria
    ASM, // American Samoa
    AND, // Andorra
    AGO, // Angola
    AIA, // Anguilla
    ATA, // Antarctica
    ATG, // Antigua and Barbuda
    ARG, // Argentina
    ARM, // Armenia
    ABW, // Aruba
    AUS, // Australia
    AUT, // Austria
    AZE, // Azerbaijan
    BHS, // Bahamas
    BHR, // Bahrain
    BGD, // Bangladesh
    BRB, // Barbados
    BLR, // Belarus
    BEL, // Belgium
    BLZ, // Belize
    BEN, // Benin
    BMU, // Bermuda
    BTN, // Bhutan
    BOL, // Bolivia (Plurinational State of)
    BES, // Bonaire, Sint Eustatius and Saba
    BIH, // Bosnia and Herzegovina
    BWA, // Botswana
    BVT, // Bouvet Island
    BRA, // Brazil
    IOT, // British Indian Ocean Territory
    BRN, // Brunei Darussalam
    BGR, // Bulgaria
    BFA, // Burkina Faso
    BDI, // Burundi
    CPV, // Cabo Verde
    KHM, // Cambodia
    CMR, // Cameroon
    CAN, // Canada
    CYM, // Cayman Islands
    CAF, // Central African Republic
    TCD, // Chad
    CHL, // Chile
    CHN, // China
    CXR, // Christmas Island
    CCK, // Cocos (Keeling) Islands
    COL, // Colombia
    COM, // Comoros
    COG, // Congo
    COD, // Congo (Democratic Republic of the)
    COK, // Cook Islands
    CRI, // Costa Rica
    CIV, // Côte d'Ivoire
    HRV, // Croatia
    CUB, // Cuba
    CUW, // Curaçao
    CYP, // Cyprus
    CZE, // Czechia
    DNK, // Denmark
    DJI, // Djibouti
    DMA, // Dominica
    DOM, // Dominican Republic
    ECU, // Ecuador
    EGY, // Egypt
    SLV, // El Salvador
    GNQ, // Equatorial Guinea
    ERI, // Eritrea
    EST, // Estonia
    SWZ, // Eswatini
    ETH, // Ethiopia
    FLK, // Falkland Islands (Malvinas)
    FRO, // Faroe Islands
    FJI, // Fiji
    FIN, // Finland
    FRA, // France
    GUF, // French Guiana
    PYF, // French Polynesia
    ATF, // French Southern Territories
    GAB, // Gabon
    GMB, // Gambia
    GEO, // Georgia
    DEU, // Germany
    GHA, // Ghana
    GIB, // Gibraltar
    GRC, // Greece
    GRL, // Greenland
    GRD, // Grenada
    GLP, // Guadeloupe
    GUM, // Guam
    GTM, // Guatemala
    GGY, // Guernsey
    GIN, // Guinea
    GNB, // Guinea-Bissau
    GUY, // Guyana
    HTI, // Haiti
    HMD, // Heard Island and McDonald Islands
    VAT, // Holy See
    HND, // Honduras
    HKG, // Hong Kong
    HUN, // Hungary
    ISL, // Iceland
    IND, // India
    IDN, // Indonesia
    IRN, // Iran (Islamic Republic of)
    IRQ, // Iraq
    IRL, // Ireland
    IMN, // Isle of Man
    ISR, // Israel
    ITA, // Italy
    JAM, // Jamaica
    JPN, // Japan
    JEY, // Jersey
    JOR, // Jordan
    KAZ, // Kazakhstan
    KEN, // Kenya
    KIR, // Kiribati
    PRK, // Korea (Democratic People's Republic of)
    KOR, // Korea (Republic of)
    KWT, // Kuwait
    KGZ, // Kyrgyzstan
    LAO, // Lao People's Democratic Republic
    LVA, // Latvia
    LBN, // Lebanon
    LSO, // Lesotho
    LBR, // Liberia
    LBY, // Libya
    LIE, // Liechtenstein
    LTU, // Lithuania
    LUX, // Luxembourg
    MAC, // Macao
    MDG, // Madagascar
    MWI, // Malawi
    MYS, // Malaysia
    MDV, // Maldives
    MLI, // Mali
    MLT, // Malta
    MHL, // Marshall Islands
    MTQ, // Martinique
    MRT, // Mauritania
    MUS, // Mauritius
    MYT, // Mayotte
    MEX, // Mexico
    FSM, // Micronesia (Federated States of)
    MDA, // Moldova (Republic of)
    MCO, // Monaco
    MNG, // Mongolia
    MNE, // Montenegro
    MSR, // Montserrat
    MAR, // Morocco
    MOZ, // Mozambique
    MMR, // Myanmar
    NAM, // Namibia
    NRU, // Nauru
    NPL, // Nepal
    NLD, // Netherlands
    NCL, // New Caledonia
    NZL, // New Zealand
    NIC, // Nicaragua
    NER, // Niger
    NGA, // Nigeria
    NIU, // Niue
    NFK, // Norfolk Island
    MKD, // North Macedonia
    MNP, // Northern Mariana Islands
    NOR, // Norway
    OMN, // Oman
    PAK, // Pakistan
    PLW, // Palau
    PSE, // Palestine, State of
    PAN, // Panama
    PNG, // Papua New Guinea
    PRY, // Paraguay
    PER, // Peru
    PHL, // Philippines
    PCN, // Pitcairn
    POL, // Poland
    PRT, // Portugal
    PRI, // Puerto Rico
    QAT, // Qatar
    ROU, // Romania
    RUS, // Russian Federation
    RWA, // Rwanda
    REU, // Réunion
    BLM, // Saint Barthélemy
    SHN, // Saint Helena, Ascension and Tristan da Cunha
    KNA, // Saint Kitts and Nevis
    LCA, // Saint Lucia
    MAF, // Saint Martin (French part)
    SPM, // Saint Pierre and Miquelon
    VCT, // Saint Vincent and the Grenadines
    WSM, // Samoa
    SMR, // San Marino
    STP, // Sao Tome and Principe
    SAU, // Saudi Arabia
    SEN, // Senegal
    SRB, // Serbia
    SYC, // Seychelles
    SLE, // Sierra Leone
    SGP, // Singapore
    SXM, // Sint Maarten (Dutch part)
    SVK, // Slovakia
    SVN, // Slovenia
    SLB, // Solomon Islands
    SOM, // Somalia
    ZAF, // South Africa
    SGS, // South Georgia and the South Sandwich Islands
    SSD, // South Sudan
    ESP, // Spain
    LKA, // Sri Lanka
    SDN, // Sudan
    SUR, // Suriname
    SJM, // Svalbard and Jan Mayen
    SWE, // Sweden
    CHE, // Switzerland
    SYR, // Syrian Arab Republic
    TWN, // Taiwan, Province of China
    TJK, // Tajikistan
    TZA, // Tanzania, United Republic of
    THA, // Thailand
    TLS, // Timor-Leste
    TGO, // Togo
    TKL, // Tokelau
    TON, // Tonga
    TTO, // Trinidad and Tobago
    TUN, // Tunisia
    TUR, // Turkey
    TKM, // Turkmenistan
    TCA, // Turks and Caicos Islands
    TUV, // Tuvalu
    UGA, // Uganda
    UKR, // Ukraine
    ARE, // United Arab Emirates
    GBR, // United Kingdom of Great Britain and Northern Ireland
    USA, // United States of America
    URY, // Uruguay
    UZB, // Uzbekistan
    VUT, // Vanuatu
    VEN, // Venezuela (Bolivarian Republic of)
    VNM, // Viet Nam
    WLF, // Wallis and Futuna
    ESH, // Western Sahara
    YEM, // Yemen
    ZMB, // Zambia
    ZWE, // Zimbabwe
}

#[derive(Debug, Clone)]
pub struct RequestMetaData {
    pub device_id: String,
    pub request_id: String,
    pub domain_uri: String,
}

impl FromRequest for RequestMetaData {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let value = req.extensions().get::<RequestMetaData>().cloned();

        let result = match value {
            Some(data) => Ok(data),
            None => Err(ErrorInternalServerError(
                RequestMetaError::ValidationStringError(
                    "Something went wrong while setting meta data".to_string(),
                ),
            )),
        };

        ready(result)
    }
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, Clone, PartialEq)]
#[sqlx(type_name = "kyc_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum KycStatus {
    Pending,
    OnHold,
    Rejected,
    Completed,
}

#[derive(Debug)]
struct RetryPolicy {
    max_retries: u32,
    backoff_value: f64,
}

impl RetryPolicy {
    fn new() -> Self {
        RetryPolicy {
            max_retries: 3,
            backoff_value: 5.0,
        }
    }
}

#[derive(Debug)]
pub struct NetworkResponse {
    status_code: u16,
    body: String,
}
impl NetworkResponse {
    pub fn get_body(&self) -> &str {
        &self.body
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("Request error: {0}")]
    Request(reqwest::Error),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Request timed out")]
    Timeout,
}

impl From<reqwest::Error> for NetworkError {
    fn from(err: reqwest::Error) -> Self {
        NetworkError::Request(err)
    }
}

#[derive(Debug)]
pub struct NetworkCall {
    pub client: Client,
}

impl NetworkCall {
    #[tracing::instrument(name = "Async Post Call", skip(), fields())]
    pub async fn async_post_call(
        &self,
        url: &str,
        payload: Option<&str>,
        headers: Option<HashMap<&str, &str>>,
    ) -> Result<NetworkResponse, NetworkError> {
        let mut req_headers = header::HeaderMap::new();
        if let Some(headers) = headers {
            for (key, value) in headers {
                req_headers.insert(
                    header::HeaderName::from_bytes(key.as_bytes()).unwrap(),
                    header::HeaderValue::from_str(value).unwrap(),
                );
            }
        }

        if payload.is_some() {
            req_headers.insert(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static("application/json"),
            );
        }

        let body = payload.unwrap_or_default().to_string();
        let response = self
            .client
            .post(url)
            .headers(req_headers)
            .timeout(Duration::from_secs(5))
            .body(body)
            .send()
            .await?;

        let status_code = response.status();
        let body = response.text().await?;
        print!("Response: {}", body);
        Ok(NetworkResponse {
            status_code: status_code.as_u16(),
            body,
        })
    }
    #[tracing::instrument(name = "Async Post Call With Retry", skip(), fields())]
    pub async fn async_post_call_with_retry(
        &self,
        url: &str,
        payload: Option<&str>,
        headers: Option<HashMap<&str, &str>>,
    ) -> Result<Value, NetworkError> {
        let retry_policy = RetryPolicy::new();
        let mut response: Option<Value> = None;
        let mut current_backoff = 1.0;

        // let start_time = Instant::now();
        for current_retry in 0..retry_policy.max_retries {
            tracing::info!("Retry attempt {}...", current_retry);
            match self.async_post_call(url, payload, headers.to_owned()).await {
                Ok(network_response) => {
                    if network_response.status_code > 500 {
                        let error_message = network_response.body;
                        return Err(NetworkError::Validation(error_message));
                    }

                    match serde_json::from_str(&network_response.body) {
                        Ok(value) => response = Some(value),
                        Err(err) => {
                            tracing::error!("Deserialization error: {}", err);
                            return Err(NetworkError::Validation(format!(
                                "Failed to deserialize response: {}",
                                err
                            )));
                        }
                    }
                    break;
                }
                Err(NetworkError::Timeout) => {
                    tracing::warn!("Request timed out. Retrying...");
                }
                Err(NetworkError::Request(err)) => {
                    tracing::error!("Request error: {}", err);
                }
                Err(NetworkError::Validation(validation_error)) => {
                    tracing::error!("Validation error: {}", validation_error);
                }
            }

            // let elapsed_time = start_time.elapsed().as_secs_f64();
            // if elapsed_time > retry_policy.max_retries as f64 * retry_policy.backoff_value {
            //     tracing::warn!("Maximum retry attempts reached.");
            //     break;
            // }

            sleep(Duration::from_secs_f64(current_backoff)).await;
            current_backoff *= retry_policy.backoff_value;
        }

        response
            .ok_or_else(|| NetworkError::Validation("No successful response received".to_string()))
    }
}

#[derive(Debug, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "ondc_np_fee_type", rename_all = "snake_case")]
pub enum FeeType {
    Percent,
    Amount,
}

impl std::fmt::Display for FeeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            FeeType::Percent => "percent",
            FeeType::Amount => "amount",
        };

        write!(f, "{}", s)
    }
}

#[derive(Debug)]
pub struct RegisteredNetworkParticipant {
    pub code: String,
    pub name: String,
    pub logo: String,
    pub signing_key: Secret<String>,
    pub id: Uuid,
    pub subscriber_id: String,
    pub subscriber_uri: String,
    pub long_description: String,
    pub short_description: String,
    pub fee_type: FeeType,
    pub fee_value: BigDecimal,
    pub unique_key_id: String,
}

#[derive(Debug, Serialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "network_participant_type", rename_all = "snake_case")]
pub enum ONDCNPType {
    Buyer,
    Seller,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WebSocketParam {
    pub user_id: Uuid,
    pub business_id: Uuid,
    pub device_id: String,
}

pub trait WSKeyTrait {
    fn get_key(&self) -> String;
}

impl WSKeyTrait for WebSocketParam {
    fn get_key(&self) -> String {
        format!("{}#{}#{}", self.user_id, self.business_id, self.device_id)
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone)]
#[serde(rename_all = "UPPERCASE")]
#[sqlx(type_name = "ondc_network_participant_type", rename_all = "UPPERCASE")]
pub enum ONDCNetworkType {
    Bap,
    Bpp,
}

#[derive(Debug)]
pub struct ONDCAuthParams {
    pub created_time: i64,
    pub expires_time: i64,
    pub subscriber_id: String,
    pub uk_id: String,
    pub algorithm: String,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, sqlx::Type)]
#[sqlx(type_name = "currency_code_type", rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum CurrencyType {
    Inr,
}

impl Display for CurrencyType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &format!("{:?}", self).to_lowercase())
    }
}
