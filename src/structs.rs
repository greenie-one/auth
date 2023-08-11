use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{database::mongo::UserModel, services::signup::ValidationType};

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationData {
    pub validation_type: ValidationType,
    pub user: UserModel,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    pub email: Option<String>,
    pub sub: String,
    pub iss: String,
    pub session_id: String,
    pub roles: Vec<String>,
    pub iat: u64,
    pub is_refresh: Option<bool>,
    pub exp: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export)]
pub struct AccessTokenResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "accessToken")]
    #[ts(optional)]
    pub access_token: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "refreshToken")]
    #[ts(optional)]
    pub refresh_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChangePasswordValidationData {
    pub otp: String,
    pub user_id: String,
}

#[derive(Debug)]
pub struct WebResponseErrorCustom {
    pub msg: String,
    pub status: u16,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GenericError<'a> {
    pub message: String,
    pub status: u16,
    pub code: &'a str,
}

#[derive(Debug, Serialize, Deserialize, TS)]
pub struct ProfileHints {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct OAuthLoginResponse {
    #[serde(rename = "accessToken")]
    #[ts(optional)]
    pub access_token: Option<String>,

    #[serde(rename = "refreshToken")]
    #[ts(optional)]
    pub refresh_token: Option<String>,

    #[ts(inline)]
    pub profile_hints: ProfileHints,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct OAuthRedirectUriResponse {
    #[serde(rename = "redirectUrl")]
    pub redirect_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct ValidationIdResponse {
    #[serde(rename = "validationId")]
    pub validation_id: String,
}

pub type SignUpResponse = ValidationIdResponse;
pub type LoginResponse = ValidationIdResponse;
pub type ForgotPasswordResponse = ValidationIdResponse;
