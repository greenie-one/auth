use serde::Deserialize;
use validator_derive::Validate;

#[derive(Debug, Validate, Clone, Deserialize)]
pub struct RefreshTokenDto {
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}
