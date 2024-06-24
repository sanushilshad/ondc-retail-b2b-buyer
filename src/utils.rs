use crate::constants::AUTHORIZATION_PATTERN;
use crate::schemas::ONDCAuthParams;
use actix_web::dev::Payload;
use actix_web::web;
pub fn get_ondc_params_from_header(header: &str) -> Result<ONDCAuthParams, anyhow::Error> {
    let captures = AUTHORIZATION_PATTERN
        .captures(header)
        .ok_or_else(|| anyhow::anyhow!("Invalid Authorization Header"))?;

    let groups: Vec<String> = captures
        .iter()
        .skip(1)
        .filter_map(|m| m.map(|m| m.as_str().to_owned()))
        .collect();

    if groups.len() != 6 {
        return Err(anyhow::anyhow!(
            "Invalid number of captured groups in Authorization Token"
        ));
    }

    let created_time = groups[3]
        .parse::<i64>()
        .map_err(|err| anyhow::anyhow!("Invalid created time format: {}", err))?;
    let expires_time = groups[4]
        .parse::<i64>()
        .map_err(|err| anyhow::anyhow!("Invalid expired time format: {}", err))?;
    let subscriber_id = groups[0].clone();
    let uk_id = groups[1].clone();
    let algorithm = groups[2].clone();
    let signature = groups[5].clone();

    Ok(ONDCAuthParams {
        created_time,
        expires_time,
        subscriber_id,
        uk_id,
        algorithm,
        signature,
    })
}

pub fn bytes_to_payload(buf: web::Bytes) -> Payload {
    let (_, mut pl) = actix_http::h1::Payload::create(true);
    pl.unread_data(buf);
    Payload::from(pl)
}
