use crate::{
    database::mongo::MONGO_DB_INSTANCE,
    error::{Error, ErrorEnum},
    structs::AccessTokenResponse,
};

use super::token::{create_token, decode_token};

pub async fn get_refreshed_tokens(refresh_token: &str) -> Result<AccessTokenResponse, Error> {
    let claims = decode_token(refresh_token)?;
    if claims.is_refresh.unwrap_or(false) {
        return get_new_tokens(claims.sub).await;
    }
    return Err(ErrorEnum::InvalidRefreshToken.into());
}

async fn get_new_tokens(user_id: String) -> Result<AccessTokenResponse, Error> {
    let database = MONGO_DB_INSTANCE.get().await;
    let user = database.find_user(None, None, Some(user_id)).await?;

    if user.is_none() {
        return Err(ErrorEnum::UnAuthorized.into());
    }

    let mut resp = create_token(user.unwrap())?;
    resp.refresh_token = None;

    Ok(resp)
}
