use std::{
    env, fs,
    time::{SystemTime, UNIX_EPOCH},
};

use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};

use crate::{
    database::mongo::UserModel,
    error::{Error, ErrorEnum},
    structs::{AccessTokenResponse, TokenClaims},
};

use lazy_static::lazy_static;

lazy_static! {
    static ref TOKEN_KEYS: (EncodingKey, DecodingKey) = get_keys();
}

fn get_keys() -> (EncodingKey, DecodingKey) {
    let mut private_key_pem = env::var("JWT_PRIVATE_KEY").map(|v| v.as_bytes().to_vec());
    let mut public_key_pem = env::var("JWT_PUBLIC_KEY").map(|v| v.as_bytes().to_vec());

    if private_key_pem.is_err() {
        private_key_pem = Ok(fs::read("./keys/local/private.pem").unwrap());
    }

    if public_key_pem.is_err() {
        public_key_pem = Ok(fs::read("./keys/local/public.pem").unwrap());
    }

    let private_key = EncodingKey::from_rsa_pem(&private_key_pem.unwrap()).unwrap();
    let public_key = DecodingKey::from_rsa_pem(&public_key_pem.unwrap()).unwrap();

    (private_key, public_key)
}

pub fn decode_token(token: &str) -> Result<TokenClaims, Error> {
    let validation = Validation::new(Algorithm::RS256);
    let token_claims: TokenData<TokenClaims> = decode(token, &TOKEN_KEYS.1, &validation)?;

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    if token_claims.claims.exp < now {
        return Err(ErrorEnum::TokenExpired.into());
    }

    Ok(token_claims.claims)
}

pub fn create_token(user: UserModel) -> Result<AccessTokenResponse, Error> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let expiry = now + 24 * 60 * 60;
    let access_claims = TokenClaims {
        email: user.email,
        iss: "greenie.one".to_owned(),
        session_id: "".to_owned(),
        roles: user.roles,
        iat: now,
        is_refresh: None,
        sub: user._id.unwrap().to_string(),
        exp: expiry,
    };

    let mut refresh_claims = access_claims.clone();
    refresh_claims.is_refresh = Some(true);

    let header = Header {
        alg: Algorithm::RS256,
        ..Default::default()
    };

    let access_token = encode(&header, &access_claims, &TOKEN_KEYS.0)?;
    let refresh_token = encode(&header, &refresh_claims, &TOKEN_KEYS.0)?;

    Ok(AccessTokenResponse {
        access_token: Some(access_token),
        refresh_token: Some(refresh_token),
    })
}
