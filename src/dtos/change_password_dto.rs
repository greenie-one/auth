use serde::Deserialize;

use ts_rs::TS;
use validator_derive::Validate;

#[derive(Debug, Validate, Clone, Deserialize, TS)]
#[ts(export)]
pub struct ValidateForgotPasswordDto {
    #[serde(rename = "validationId")]
    pub validation_id: String,

    pub otp: String,

    #[serde(rename = "newPassword")]
    pub new_password: String,
}

#[derive(Debug, Validate, Clone, Deserialize, TS)]
#[ts(export)]
pub struct ForgotPasswordDto {
    pub email: String,
}

#[derive(Debug, Validate, Clone, Deserialize, TS)]
#[ts(export)]
pub struct ChangePasswordDto {
    #[serde(rename = "currentPassword")]
    pub current_password: Option<String>,

    #[serde(rename = "newPassword")]
    pub new_password: String,
}

impl From<ValidateForgotPasswordDto> for ChangePasswordDto {
    fn from(value: ValidateForgotPasswordDto) -> Self {
        Self {
            current_password: None,
            new_password: value.new_password,
        }
    }
}
