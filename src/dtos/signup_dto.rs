use std::borrow::Cow;

use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use serde::Deserializer;
use ts_rs::TS;
use validator::ValidationError;
use validator_derive::Validate;

lazy_static! {
    static ref MOBILE_REGEX: Regex =
        Regex::new(r"^(?:(?:\+|0{0,2})91(\s*[\-]\s*)?|[0]?)?[6789]\d{9}$").unwrap();
}

#[derive(Debug, Validate, Clone, Deserialize, TS)]
#[ts(export)]
#[validate(schema(function = "validate_create_user_dto", skip_on_field_errors = false))]
pub struct CreateUserDto {
    #[validate(email)]
    #[ts(optional)]
    pub email: Option<String>,

    #[serde(rename = "mobileNumber")]
    #[serde(default)]
    #[serde(deserialize_with = "sanitize_mobile")]
    #[validate(regex = "MOBILE_REGEX")]
    #[ts(optional)]
    pub mobile_number: Option<String>,

    #[ts(optional)]
    pub password: Option<String>,
}

fn validate_create_user_dto(dto: &CreateUserDto) -> Result<(), ValidationError> {
    if dto.email.is_some() && dto.password.is_none() {
        let mut err = ValidationError::new("missing_pass");
        err.message = Some(Cow::Owned("Password should not be empty".to_owned()));
        return Err(err);
    }
    Ok(())
}

fn sanitize_mobile<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Deserialize::deserialize(deserializer)?;

    if s.is_some() {
        let unwrapped = s.clone().unwrap();
        if !unwrapped.starts_with("+") {
            return Ok(Some(format!("+91{}", unwrapped)));
        }
    }
    Ok(s)
}
