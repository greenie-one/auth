use serde::Deserialize;

use validator_derive::Validate;

#[derive(Debug, Validate, Clone, Deserialize)]
pub struct ValidateForgotPasswordDto {
    pub validation_id: String,
    pub otp: String,
    pub current_password: Option<String>,
    pub new_password: String,
}

#[derive(Debug, Validate, Clone, Deserialize)]
pub struct ForgotPasswordDto {
    pub email: String,
}

#[derive(Debug, Validate, Clone, Deserialize)]
pub struct ChangePasswordDto {
    pub current_password: Option<String>,
    pub new_password: String,
}

impl From<ValidateForgotPasswordDto> for ChangePasswordDto {
    fn from(value: ValidateForgotPasswordDto) -> Self {
        Self {
            current_password: value.current_password,
            new_password: value.new_password,
        }
    }
}
