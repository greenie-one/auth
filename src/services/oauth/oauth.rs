extern crate jsonwebkey as jwk;
extern crate jsonwebtoken as jwt;

use async_trait::async_trait;
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

use crate::error::{Error, ErrorEnum};

use super::{google::GoogleProvider, linkedin::LinkedInProvider};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileHints {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthLoginResponse {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub profile_hints: ProfileHints,
}

#[async_trait]
#[enum_dispatch]
pub trait OAuthProviders {
    fn get_redirect_uri(&self) -> Result<String, Error>;
    async fn handle_login(&self, url: String) -> Result<OAuthLoginResponse, Error>;
}

#[enum_dispatch(OAuthProviders)]
pub enum Providers {
    LinkedIn(LinkedInProvider),
    Google(GoogleProvider),
}

pub fn get_provider(provider_slug: &str) -> Result<Providers, Error> {
    match provider_slug {
        "linkedin" => Ok(Providers::LinkedIn(LinkedInProvider)),
        "google" => Ok(Providers::Google(GoogleProvider)),
        &_ => Err(ErrorEnum::OAuthProviderNotFound.into()),
    }
}
