use serde::Deserialize;
use validator_derive::Validate;

#[derive(Debug, Validate, Clone, Deserialize)]
pub struct ResendOTPDto {
    #[serde(rename = "validationId")]
    pub validation_id: String,
}
