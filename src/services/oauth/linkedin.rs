use async_trait::async_trait;
use url::Url;

use crate::error::{Error, ErrorEnum};

use super::oauth::{OAuthProviders, OAuthLoginResponse};

pub struct LinkedInProvider;

#[async_trait]
impl OAuthProviders for LinkedInProvider {
    fn get_redirect_uri(&self) -> Result<String, Error> {
        let mut url = Url::parse("https://www.linkedin.com/oauth/v2/authorization")?;
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("response_type", "code");
            query.append_pair("client_id", &std::env::var("LINKEDIN_CLIENT_ID")?);
            query.append_pair("redirect_uri", &std::env::var("LINKEDIN_REDIRECT_URI")?);
            query.append_pair("scope", "openid email profile r_liteprofile");
        }

        Ok(url.to_string())
    }

    async fn handle_login(&self, _url: String) -> Result<OAuthLoginResponse,Error> {
        Err(ErrorEnum::NotYetImplemented.into())
    }
}