use serde::Deserialize;
use ts_rs::TS;
use validator_derive::Validate;

#[derive(Debug, Validate, Clone, Deserialize, TS)]
#[ts(export)]
pub struct ResendOTPDto {
    #[serde(rename = "validationId")]
    pub validation_id: String,
}
