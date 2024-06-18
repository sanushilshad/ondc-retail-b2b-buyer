pub const ONDC_TTL: &str = "PT30";
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
  pub static ref AUTHORIZATION_PATTERN: Regex = Regex::new(
      r#"^Signature keyId=\"(.+)\|(.+)\|.*\",algorithm=\"(ed25519)\",\s*created=\"(\d+)\"\s*,\s*expires=\"(\d+)\"\s*,\s*headers\s*=\"\(created\)\s*\(expires\)\s*digest\",\s*signature=\"(.*)\"\s*$"#
  ).expect("Failed to compile regex pattern");
}
