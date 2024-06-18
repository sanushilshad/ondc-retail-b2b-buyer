use anyhow::Error;

use crate::constants::AUTHORIZATION_PATTERN;
use crate::schemas::ONDCAuthParams;

// pub fn get_ondc_params_from_header(header: &str) -> Result<ONDCAuthParams, Error> {
//     // Attempt to match the header using the regex pattern
//     let captures = AUTHORIZATION_PATTERN
//         .captures(header)
//         .ok_or_else(|| Err(anyhow::anyhow!("Invalid Authrization Header")))?;

//     // Collect the captured groups into a vector
//     let groups: Vec<_> = captures
//         .iter()
//         .skip(1) // Skip the full match
//         .map(|m| m.map(|m| m.as_str().to_owned()))
//         .collect();

//     // Ensure we have the expected number of groups
//     if groups.len() != 6 {
//         return Err(anyhow::anyhow!(
//             "Invalid number of captured groups in Authorization Token"
//         ));
//     }

//     let created_time = groups[3]
//         .parse::<i64>()
//         .map_err(|_| Err(anyhow::anyhow!("Invalid created time format")))?;
//     let expires_time = groups[4]
//         .parse::<i64>()
//         .map_err(|_| Err(anyhow::anyhow!("Invalid expires time format")))?;
//     let subscriber_id = groups[0].clone();
//     let uk_id = groups[1].clone();
//     let algorithm = groups[2].clone();
//     let signature = groups[5].clone();

//     // Construct and return the ONDCAuthParams struct
//     Ok(ONDCAuthParams {
//         created_time,
//         expires_time,
//         subscriber_id,
//         uk_id,
//         algorithm,
//         signature,
//     })
// }
