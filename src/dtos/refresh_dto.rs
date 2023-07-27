use serde::Deserialize;
use ts_rs::TS;
use validator_derive::Validate;

#[derive(Debug, Validate, Clone, Deserialize, TS)]
#[ts(export)]
pub struct RefreshTokenDto {
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}
