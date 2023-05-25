use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessTokenResponse {
    #[serde(rename = "accessToken")]
    pub access_token: String,

    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}
