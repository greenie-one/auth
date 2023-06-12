use std::{
    borrow::Cow,
    collections::HashMap,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use jsonwebkey::JsonWebKey;
use jsonwebtoken::{decode, TokenData, Validation};
use serde::Deserialize;
use url::Url;

use crate::{
    database::mongo::UserModel,
    error::{Error, ErrorEnum},
    services::{signup::insert_user, token::create_token},
};

use super::oauth::{OAuthLoginResponse, OAuthProviders, ProfileHints};

#[derive(Debug, Deserialize)]
pub struct GoogleAccessTokenResponse {
    pub access_token: Option<String>,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    pub token_type: Option<String>,
    pub id_token: Option<String>,

    pub error: Option<String>,
    pub error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GoogleAccessTokenClaims {
    exp: u64,
    email:  Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
}

pub struct GoogleProvider;

impl GoogleProvider {
    fn decode_token(&self, token: &str) -> Result<GoogleAccessTokenClaims, Error> {
        let jwks = reqwest::blocking::get("https://www.googleapis.com/oauth2/v3/certs")?
            .json::<serde_json::Value>()?;
        let jwks = jwks
            .as_object()
            .unwrap()
            .get("keys")
            .unwrap()
            .as_array()
            .unwrap()[0]
            .to_string();
        let jwk: JsonWebKey = jwks.parse().unwrap();

        let alg = jwk.algorithm.unwrap();
        let validation = Validation::new(alg.into());

        let claims: TokenData<GoogleAccessTokenClaims> =
            decode(token, &jwk.key.to_decoding_key(), &validation)?;

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        if now > claims.claims.exp {
            return Err(ErrorEnum::TokenExpired.into());
        }

        Ok(claims.claims)
    }

    async fn get_access_token_claims(
        &self,
        auth_code: Cow<'_, str>,
    ) -> Result<GoogleAccessTokenClaims, Error> {
        let client = reqwest::blocking::Client::builder().build()?;

        let mut params = HashMap::new();
        params.insert("grant_type", "authorization_code");
        params.insert("code", &auth_code);

        let binding = std::env::var("GOOGLE_CLIENT_ID")?;
        params.insert("client_id", &binding);

        let binding = std::env::var("GOOGLE_CLIENT_SECRET")?;
        params.insert("client_secret", &binding);

        let binding = std::env::var("GOOGLE_REDIRECT_URI")?;
        params.insert("redirect_uri", &binding);

        let resp = client
            .post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send()?
            .json::<GoogleAccessTokenResponse>()?;

        if resp.error.is_some() {
            return Err(ErrorEnum::OAuthFailed(format!(
                "{}: {}",
                resp.error.unwrap(),
                resp.error_description.unwrap_or_default()
            ))
            .into());
        }

        if resp.id_token.is_some() {
            return self.decode_token(&resp.id_token.unwrap());
        }

        Err(ErrorEnum::OAuthFailed("Missing id token in response".to_string()).into())
    }
}

#[async_trait]
impl OAuthProviders for GoogleProvider {
    fn get_redirect_uri(&self) -> Result<String, Error> {
        let mut url = Url::parse("https://accounts.google.com/o/oauth2/v2/auth")?;
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("response_type", "code");
            query.append_pair("client_id", &std::env::var("GOOGLE_CLIENT_ID")?);
            query.append_pair("redirect_uri", &std::env::var("GOOGLE_REDIRECT_URI")?);
            query.append_pair("scope", "https://www.googleapis.com/auth/userinfo.profile https://www.googleapis.com/auth/userinfo.email");
            query.append_pair("access_type", "offline");
            query.append_pair("prompt", "consent");
        }

        Ok(url.to_string())
    }

    async fn handle_login(&self, url: String) -> Result<OAuthLoginResponse, Error> {
        let url = Url::from_str(&url)?;
        let code_opt = url.query_pairs().find(|v| v.0 == "code");

        if code_opt.is_none() {
            return Err(ErrorEnum::ValidationError("code cannot be empty".to_string()).into())
        }

        let code = code_opt.unwrap();
        let access_token_claims = self.get_access_token_claims(code.1).await?;

        let mut user = UserModel {
            _id: None,
            email: access_token_claims.email,
            mobile_number: None,
            password: None,
            roles: vec!["default".to_string()],
        };

        let insert_user_resp = insert_user(user.clone()).await;
        if insert_user_resp.is_err() {
            user = match insert_user_resp.unwrap_err() {
                Error::DefinedError(e) => {
                    match e {
                        ErrorEnum::UserAlreadyExists(u) => {
                            Ok(u)
                        },
                        _ => Err(Error::DefinedError(e))
                    }
                },
             e => Err(e)
            }?;
        }

        let token = create_token(user)?;

        Ok(OAuthLoginResponse {
            access_token: token.access_token,
            refresh_token: token.refresh_token,
            profile_hints: ProfileHints {
                first_name: access_token_claims.given_name,
                last_name: access_token_claims.family_name
            }
        })
    }
}
