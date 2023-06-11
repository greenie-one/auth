use std::{
    fmt::{self, Display, Formatter},
    sync::{MutexGuard, PoisonError},
    time::SystemTimeError,
};

use ntex::{
    http::{header::ToStrError, StatusCode},
    web::{self, HttpRequest, WebResponseError},
};
use redis::RedisError;

use serde_json::{json, Error as JsonError};
use validator::ValidationErrors;

use crate::database::redis::Redis;

#[derive(Debug)]
pub struct WebResponseErrorCustom {
    msg: String,
    status: u16,
}

#[derive(Debug)]
pub enum Error {
    ValidationErrors(ValidationErrors),
    MongoError(mongodb::error::Error),
    RedisError(RedisError),
    JsonError(JsonError),
    BcryptError(bcrypt::BcryptError),
    SystemTimeError(SystemTimeError),
    JWTError(jsonwebtoken::errors::Error),
    ToStrError(ToStrError),
    WebResponseErrorCustom(WebResponseErrorCustom),
}

impl Error {
    pub fn new(msg: &str, status: u16) -> Self {
        Self::WebResponseErrorCustom(WebResponseErrorCustom {
            msg: msg.to_owned(),
            status,
        })
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl WebResponseError for Error {
    fn error_response(&self, _: &HttpRequest) -> web::HttpResponse {
        println!("{:?}", self);

        let (err_json, status) = match self {
            Error::MongoError(_)
            | Error::RedisError(_)
            | Error::BcryptError(_)
            | Error::SystemTimeError(_)
            | Error::JsonError(_) => (json!({ "error": "Internal server error" }), 500),

            Error::JWTError(_) => (json!("Unauthorized"), 401),

            Error::ValidationErrors(e) => (json!({"error": e.to_string()}), 400),

            Error::WebResponseErrorCustom(e) => (json!({ "error": e.msg }), e.status),
            Error::ToStrError(e) => (json!({"error": e.to_string()}), 400),
        };
        web::HttpResponse::build(StatusCode::from_u16(status).unwrap()).json(&err_json)
    }
}

impl From<ValidationErrors> for Error {
    fn from(value: ValidationErrors) -> Self {
        Error::ValidationErrors(value)
    }
}

impl From<mongodb::error::Error> for Error {
    fn from(value: mongodb::error::Error) -> Self {
        Error::MongoError(value)
    }
}

impl From<RedisError> for Error {
    fn from(value: RedisError) -> Self {
        Error::RedisError(value)
    }
}

impl From<JsonError> for Error {
    fn from(value: JsonError) -> Self {
        Error::JsonError(value)
    }
}

impl From<bcrypt::BcryptError> for Error {
    fn from(value: bcrypt::BcryptError) -> Self {
        Error::BcryptError(value)
    }
}

impl From<SystemTimeError> for Error {
    fn from(value: SystemTimeError) -> Self {
        Error::SystemTimeError(value)
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        Error::JWTError(value)
    }
}

impl From<ToStrError> for Error {
    fn from(value: ToStrError) -> Self {
        Error::ToStrError(value)
    }
}

impl From<PoisonError<MutexGuard<'_, Redis>>> for Error {
    fn from(value: PoisonError<MutexGuard<'_, Redis>>) -> Self {
        Error::WebResponseErrorCustom(WebResponseErrorCustom {
            msg: value.to_string(),
            status: 500,
        })
    }
}
