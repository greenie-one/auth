use serde::Deserialize;
use validator_derive::Validate;

#[derive(Debug, Validate, Clone, Deserialize)]
pub struct ValidateOtpDto {
    pub otp: String,

    #[serde(rename = "validationId")]
    pub validation_id: String,
}
