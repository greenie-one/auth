use std::{
    backtrace::Backtrace,
    fmt::{self, Display, Formatter},
    time::SystemTimeError,
};

use ntex::{
    http::StatusCode,
    web::{self, HttpRequest, WebResponseError},
};
use redis::RedisError;

use serde_json::{json, Error as JsonError};
use validator::ValidationErrors;

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
    // builds the actual response to send back when an error occurs
    fn error_response(&self, _: &HttpRequest) -> web::HttpResponse {
        let (err_json, status) = match self {
            Error::ValidationErrors(_)
            | Error::MongoError(_)
            | Error::RedisError(_)
            | Error::BcryptError(_)
            | Error::SystemTimeError(_)
            | Error::JWTError(_)
            | Error::JsonError(_) => (json!({ "error": "Internal server error" }), 500),

            Error::WebResponseErrorCustom(e) => (json!({ "error": e.msg }), e.status),
        };
        web::HttpResponse::build(StatusCode::from_u16(status).unwrap()).json(&err_json)
    }
}

impl From<ValidationErrors> for Error {
    fn from(value: ValidationErrors) -> Self {
        println!("{:?}", value);
        println!("{}", Backtrace::capture());
        Error::ValidationErrors(value)
    }
}

impl From<mongodb::error::Error> for Error {
    fn from(value: mongodb::error::Error) -> Self {
        println!("{:?}", value);
        println!("{}", Backtrace::capture());
        Error::MongoError(value)
    }
}

impl From<RedisError> for Error {
    fn from(value: RedisError) -> Self {
        println!("{:?}", value);
        println!("{}", Backtrace::capture());
        Error::RedisError(value)
    }
}

impl From<JsonError> for Error {
    fn from(value: JsonError) -> Self {
        println!("{:?}", value);
        println!("{}", Backtrace::capture());
        Error::JsonError(value)
    }
}

impl From<bcrypt::BcryptError> for Error {
    fn from(value: bcrypt::BcryptError) -> Self {
        println!("{:?}", value);
        println!("{}", Backtrace::capture());
        Error::BcryptError(value)
    }
}

impl From<SystemTimeError> for Error {
    fn from(value: SystemTimeError) -> Self {
        println!("{:?}", value);
        println!("{}", Backtrace::capture());
        Error::SystemTimeError(value)
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        println!("{:?}", value);
        println!("{}", Backtrace::capture());
        Error::JWTError(value)
    }
}
