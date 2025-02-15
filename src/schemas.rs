use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    str::FromStr,
    time::Duration,
};

use crate::routes::order::schemas::PaymentSettlementType;
use crate::{errors::RequestMetaError, routes::order::schemas::PaymentSettlementPhase};
use actix_http::StatusCode;
use actix_web::{error::ErrorInternalServerError, FromRequest, HttpMessage};
use bigdecimal::BigDecimal;
use futures_util::future::{ready, Ready};
use reqwest::{header, Client};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::time::sleep;
use utoipa::ToSchema;
use uuid::Uuid;
#[derive(Serialize, Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
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
            code: StatusCode::OK.as_str().to_owned(),
            data,
        }
    }

    pub fn error(message: &str, code: StatusCode, data: Option<D>) -> Self {
        Self {
            status: false,
            customer_message: String::from(message),
            code: code.as_str().to_owned(),
            data,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, Clone, PartialEq, ToSchema)]
#[sqlx(rename_all = "lowercase", type_name = "status")]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Active,
    Inactive,
    Pending,
    Archived,
}

// impl PgHasArrayType for Status {
//     fn array_type_info() -> sqlx::postgres::PgTypeInfo {
//         sqlx::postgres::PgTypeInfo::with_name("_status")
//     }
// }

#[derive(Debug, Eq, PartialEq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommunicationType {
    Type1,
}

#[derive(Debug, Deserialize, Serialize, Clone, ToSchema, sqlx::Type, PartialEq)]
#[sqlx(rename_all = "UPPERCASE", type_name = "country_code_type")]
#[serde(rename_all = "UPPERCASE")]
pub enum CountryCode {
    AFG,
    ALA,
    ALB,
    DZA,
    ASM,
    AND,
    AGO,
    AIA,
    ATA,
    ATG,
    ARG,
    ARM,
    ABW,
    AUS,
    AUT,
    AZE,
    BHS,
    BHR,
    BGD,
    BRB,
    BLR,
    BEL,
    BLZ,
    BEN,
    BMU,
    BTN,
    BOL,
    BES,
    BIH,
    BWA,
    BVT,
    BRA,
    IOT,
    BRN,
    BGR,
    BFA,
    BDI,
    CPV,
    KHM,
    CMR,
    CAN,
    CYM,
    CAF,
    TCD,
    CHL,
    CHN,
    CXR,
    CCK,
    COL,
    COM,
    COG,
    COD,
    COK,
    CRI,
    CIV,
    HRV,
    CUB,
    CUW,
    CYP,
    CZE,
    DNK,
    DJI,
    DMA,
    DOM,
    ECU,
    EGY,
    SLV,
    GNQ,
    ERI,
    EST,
    SWZ,
    ETH,
    FLK,
    FRO,
    FJI,
    FIN,
    FRA,
    GUF,
    PYF,
    ATF,
    GAB,
    GMB,
    GEO,
    DEU,
    GHA,
    GIB,
    GRC,
    GRL,
    GRD,
    GLP,
    GUM,
    GTM,
    GGY,
    GIN,
    GNB,
    GUY,
    HTI,
    HMD,
    VAT,
    HND,
    HKG,
    HUN,
    ISL,
    IND,
    IDN,
    IRN,
    IRQ,
    IRL,
    IMN,
    ISR,
    ITA,
    JAM,
    JPN,
    JEY,
    JOR,
    KAZ,
    KEN,
    KIR,
    PRK,
    KOR,
    KWT,
    KGZ,
    LAO,
    LVA,
    LBN,
    LSO,
    LBR,
    LBY,
    LIE,
    LTU,
    LUX,
    MAC,
    MDG,
    MWI,
    MYS,
    MDV,
    MLI,
    MLT,
    MHL,
    MTQ,
    MRT,
    MUS,
    MYT,
    MEX,
    FSM,
    MDA,
    MCO,
    MNG,
    MNE,
    MSR,
    MAR,
    MOZ,
    MMR,
    NAM,
    NRU,
    NPL,
    NLD,
    NCL,
    NZL,
    NIC,
    NER,
    NGA,
    NIU,
    NFK,
    MKD,
    MNP,
    NOR,
    OMN,
    PAK,
    PLW,
    PSE,
    PAN,
    PNG,
    PRY,
    PER,
    PHL,
    PCN,
    POL,
    PRT,
    PRI,
    QAT,
    ROU,
    RUS,
    RWA,
    REU,
    BLM,
    SHN,
    KNA,
    LCA,
    MAF,
    SPM,
    VCT,
    WSM,
    SMR,
    STP,
    SAU,
    SEN,
    SRB,
    SYC,
    SLE,
    SGP,
    SXM,
    SVK,
    SVN,
    SLB,
    SOM,
    ZAF,
    SGS,
    SSD,
    ESP,
    LKA,
    SDN,
    SUR,
    SJM,
    SWE,
    CHE,
    SYR,
    TWN,
    TJK,
    TZA,
    THA,
    TLS,
    TGO,
    TKL,
    TON,
    TTO,
    TUN,
    TUR,
    TKM,
    TCA,
    TUV,
    UGA,
    UKR,
    ARE,
    GBR,
    USA,
    URY,
    UZB,
    VUT,
    VEN,
    VNM,
    WLF,
    ESH,
    YEM,
    ZMB,
    ZWE,
}

impl FromStr for CountryCode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "AFG" => Ok(CountryCode::AFG),
            "ALA" => Ok(CountryCode::ALA),
            "ALB" => Ok(CountryCode::ALB),
            "DZA" => Ok(CountryCode::DZA),
            "ASM" => Ok(CountryCode::ASM),
            "AND" => Ok(CountryCode::AND),
            "AGO" => Ok(CountryCode::AGO),
            "AIA" => Ok(CountryCode::AIA),
            "ATA" => Ok(CountryCode::ATA),
            "ATG" => Ok(CountryCode::ATG),
            "ARG" => Ok(CountryCode::ARG),
            "ARM" => Ok(CountryCode::ARM),
            "ABW" => Ok(CountryCode::ABW),
            "AUS" => Ok(CountryCode::AUS),
            "AUT" => Ok(CountryCode::AUT),
            "AZE" => Ok(CountryCode::AZE),
            "BHS" => Ok(CountryCode::BHS),
            "BHR" => Ok(CountryCode::BHR),
            "BGD" => Ok(CountryCode::BGD),
            "BRB" => Ok(CountryCode::BRB),
            "BLR" => Ok(CountryCode::BLR),
            "BEL" => Ok(CountryCode::BEL),
            "BLZ" => Ok(CountryCode::BLZ),
            "BEN" => Ok(CountryCode::BEN),
            "BMU" => Ok(CountryCode::BMU),
            "BTN" => Ok(CountryCode::BTN),
            "BOL" => Ok(CountryCode::BOL),
            "BES" => Ok(CountryCode::BES),
            "BIH" => Ok(CountryCode::BIH),
            "BWA" => Ok(CountryCode::BWA),
            "BVT" => Ok(CountryCode::BVT),
            "BRA" => Ok(CountryCode::BRA),
            "IOT" => Ok(CountryCode::IOT),
            "BRN" => Ok(CountryCode::BRN),
            "BGR" => Ok(CountryCode::BGR),
            "BFA" => Ok(CountryCode::BFA),
            "BDI" => Ok(CountryCode::BDI),
            "CPV" => Ok(CountryCode::CPV),
            "KHM" => Ok(CountryCode::KHM),
            "CMR" => Ok(CountryCode::CMR),
            "CAN" => Ok(CountryCode::CAN),
            "CYM" => Ok(CountryCode::CYM),
            "CAF" => Ok(CountryCode::CAF),
            "TCD" => Ok(CountryCode::TCD),
            "CHL" => Ok(CountryCode::CHL),
            "CHN" => Ok(CountryCode::CHN),
            "CXR" => Ok(CountryCode::CXR),
            "CCK" => Ok(CountryCode::CCK),
            "COL" => Ok(CountryCode::COL),
            "COM" => Ok(CountryCode::COM),
            "COG" => Ok(CountryCode::COG),
            "COD" => Ok(CountryCode::COD),
            "COK" => Ok(CountryCode::COK),
            "CRI" => Ok(CountryCode::CRI),
            "CIV" => Ok(CountryCode::CIV),
            "HRV" => Ok(CountryCode::HRV),
            "CUB" => Ok(CountryCode::CUB),
            "CUW" => Ok(CountryCode::CUW),
            "CYP" => Ok(CountryCode::CYP),
            "CZE" => Ok(CountryCode::CZE),
            "DNK" => Ok(CountryCode::DNK),
            "DJI" => Ok(CountryCode::DJI),
            "DMA" => Ok(CountryCode::DMA),
            "DOM" => Ok(CountryCode::DOM),
            "ECU" => Ok(CountryCode::ECU),
            "EGY" => Ok(CountryCode::EGY),
            "SLV" => Ok(CountryCode::SLV),
            "GNQ" => Ok(CountryCode::GNQ),
            "ERI" => Ok(CountryCode::ERI),
            "EST" => Ok(CountryCode::EST),
            "SWZ" => Ok(CountryCode::SWZ),
            "ETH" => Ok(CountryCode::ETH),
            "FLK" => Ok(CountryCode::FLK),
            "FRO" => Ok(CountryCode::FRO),
            "FJI" => Ok(CountryCode::FJI),
            "FIN" => Ok(CountryCode::FIN),
            "FRA" => Ok(CountryCode::FRA),
            "GUF" => Ok(CountryCode::GUF),
            "PYF" => Ok(CountryCode::PYF),
            "ATF" => Ok(CountryCode::ATF),
            "GAB" => Ok(CountryCode::GAB),
            "GMB" => Ok(CountryCode::GMB),
            "GEO" => Ok(CountryCode::GEO),
            "DEU" => Ok(CountryCode::DEU),
            "GHA" => Ok(CountryCode::GHA),
            "GIB" => Ok(CountryCode::GIB),
            "GRC" => Ok(CountryCode::GRC),
            "GRL" => Ok(CountryCode::GRL),
            "GRD" => Ok(CountryCode::GRD),
            "GLP" => Ok(CountryCode::GLP),
            "GUM" => Ok(CountryCode::GUM),
            "GTM" => Ok(CountryCode::GTM),
            "GGY" => Ok(CountryCode::GGY),
            "GIN" => Ok(CountryCode::GIN),
            "GNB" => Ok(CountryCode::GNB),
            "GUY" => Ok(CountryCode::GUY),
            "HTI" => Ok(CountryCode::HTI),
            "HMD" => Ok(CountryCode::HMD),
            "VAT" => Ok(CountryCode::VAT),
            "HND" => Ok(CountryCode::HND),
            "HKG" => Ok(CountryCode::HKG),
            "HUN" => Ok(CountryCode::HUN),
            "ISL" => Ok(CountryCode::ISL),
            "IND" => Ok(CountryCode::IND),
            "IDN" => Ok(CountryCode::IDN),
            "IRN" => Ok(CountryCode::IRN),
            "IRQ" => Ok(CountryCode::IRQ),
            "IRL" => Ok(CountryCode::IRL),
            "IMN" => Ok(CountryCode::IMN),
            "ISR" => Ok(CountryCode::ISR),
            "ITA" => Ok(CountryCode::ITA),
            "JAM" => Ok(CountryCode::JAM),
            "JPN" => Ok(CountryCode::JPN),
            "JEY" => Ok(CountryCode::JEY),
            "JOR" => Ok(CountryCode::JOR),
            "KAZ" => Ok(CountryCode::KAZ),
            "KEN" => Ok(CountryCode::KEN),
            "KIR" => Ok(CountryCode::KIR),
            "PRK" => Ok(CountryCode::PRK),
            "KOR" => Ok(CountryCode::KOR),
            "KWT" => Ok(CountryCode::KWT),
            "KGZ" => Ok(CountryCode::KGZ),
            "LAO" => Ok(CountryCode::LAO),
            "LVA" => Ok(CountryCode::LVA),
            "LBN" => Ok(CountryCode::LBN),
            "LSO" => Ok(CountryCode::LSO),
            "LBR" => Ok(CountryCode::LBR),
            "LBY" => Ok(CountryCode::LBY),
            "LIE" => Ok(CountryCode::LIE),
            "LTU" => Ok(CountryCode::LTU),
            "LUX" => Ok(CountryCode::LUX),
            "MAC" => Ok(CountryCode::MAC),
            "MDG" => Ok(CountryCode::MDG),
            "MWI" => Ok(CountryCode::MWI),
            "MYS" => Ok(CountryCode::MYS),
            "MDV" => Ok(CountryCode::MDV),
            "MLI" => Ok(CountryCode::MLI),
            "MLT" => Ok(CountryCode::MLT),
            "MHL" => Ok(CountryCode::MHL),
            "MTQ" => Ok(CountryCode::MTQ),
            "MRT" => Ok(CountryCode::MRT),
            "MUS" => Ok(CountryCode::MUS),
            "MYT" => Ok(CountryCode::MYT),
            "MEX" => Ok(CountryCode::MEX),
            "FSM" => Ok(CountryCode::FSM),
            "MDA" => Ok(CountryCode::MDA),
            _ => Err("Invalid Country".to_owned()),
        }
    }
}

// impl std::fmt::Display for CountryCode {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", pascal_to_uppercase(&format!("{:?}", self)))
//     }
// }

#[derive(Debug, Clone)]
pub struct RequestMetaData {
    pub device_id: String,
    pub request_id: String,
    // pub domain_uri: String,
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

#[derive(Debug, Deserialize, sqlx::Type, Serialize, Clone, ToSchema)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "ondc_np_fee_type", rename_all = "snake_case")]
pub enum FeeType {
    Percent,
    Amount,
}
// impl PgHasArrayType for &FeeType {
//     fn array_type_info() -> sqlx::postgres::PgTypeInfo {
//         sqlx::postgres::PgTypeInfo::with_name("_ondc_np_fee_type")
//     }
// }

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
    pub signing_key: SecretString,
    pub id: i32,
    pub subscriber_id: String,
    pub subscriber_uri: String,
    pub long_description: String,
    pub short_description: String,
    pub fee_type: FeeType,
    pub fee_value: BigDecimal,
    pub unique_key_id: String,
    pub settlement_phase: PaymentSettlementPhase,
    pub settlement_type: PaymentSettlementType,
    pub bank_account_no: String,
    pub bank_ifsc_code: String,
    pub bank_beneficiary_name: String,
    pub bank_name: String,
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
    pub user_id: Option<Uuid>,
    pub business_id: Uuid,
    pub device_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, ToSchema)]
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
    Sgd,
    Aed,
    Ghs,
}

impl Display for CurrencyType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &format!("{:?}", self).to_lowercase())
    }
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, PartialEq, ToSchema)]
#[sqlx(type_name = "data_source_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum DataSource {
    PlaceOrder,
    Ondc,
    Rapidor,
}

#[derive(Debug, PartialEq)]
pub enum RequestType {
    Internal,
    External,
}

#[derive(Debug, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "series_type")]
#[sqlx(rename_all = "lowercase")]
pub enum SeriesNoType {
    Order,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, sqlx::Type)]
pub enum TimeZones {
    #[serde(rename = "Asia/Kolkata")]
    AsiaKolkata,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JWTClaims {
    pub sub: Uuid,
    pub exp: usize,
}
