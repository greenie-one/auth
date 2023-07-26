extern crate jsonwebkey as jwk;
extern crate jsonwebtoken as jwt;

use async_trait::async_trait;
use enum_dispatch::enum_dispatch;

pub use crate::structs::OAuthLoginResponse;
use crate::error::{Error, ErrorEnum};

use super::{google::GoogleProvider, linkedin::LinkedInProvider};

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
