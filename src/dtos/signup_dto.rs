use std::borrow::Cow;

use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use validator::ValidationError;
use validator_derive::Validate;

lazy_static! {
    static ref MOBILE_REGEX: Regex =
        Regex::new(r"^(?:(?:\+|0{0,2})91(\s*[\-]\s*)?|[0]?)?[789]\d{9}$").unwrap();
}

#[derive(Debug, Validate, Clone, Deserialize)]
#[validate(schema(function = "validate_create_user_dto", skip_on_field_errors = false))]
pub struct CreateUserDto {
    #[validate(email)]
    pub email: Option<String>,

    #[serde(rename = "mobileNumber")]
    #[validate(regex = "MOBILE_REGEX")]
    pub mobile_number: Option<String>,

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
