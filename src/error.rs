use std::{
    env::VarError,
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
use url::ParseError;
use validator::ValidationErrors;

use crate::{
    database::{mongo::UserModel, redis::Redis},
    structs::{GenericError, WebResponseErrorCustom},
};

#[derive(Debug)]
pub enum ErrorEnum {
    UnAuthorized,
    UserAlreadyExists(UserModel),
    InvalidValidationId,
    InvalidRefreshToken,
    UserNotFound,
    UserContactMissing,
    PasswordMismatch,
    EmailMobileEmpty,
    InvalidOTP,
    TokenExpired,
    OAuthProviderNotFound,
    OAuthFailed(String),
    NotYetImplemented,
    ValidationError(String),
}

fn get_error<'a>(val: &ErrorEnum) -> GenericError<'a> {
    match val {
        ErrorEnum::ValidationError(e) => GenericError {
            code: "GRA0000",
            message: format!("Failed to validate request: [{}]", e),
            status: 400,
        },
        ErrorEnum::UnAuthorized => GenericError {
            message: "Unauthorized".to_string(),
            status: 401,
            code: "GRA0001",
        },
        ErrorEnum::UserAlreadyExists(_) => GenericError {
            code: "GRA0003",
            message: "User already exists".to_string(),
            status: 409,
        },
        ErrorEnum::InvalidValidationId => GenericError {
            code: "GRA0004",
            message: "Invalid validation ID".to_string(),
            status: 400,
        },
        ErrorEnum::InvalidRefreshToken => GenericError {
            code: "GRA0005",
            message: "Invalid refresh token".to_string(),
            status: 400,
        },
        ErrorEnum::UserNotFound => GenericError {
            code: "GRA0008",
            message: "User not found".to_string(),
            status: 404,
        },
        ErrorEnum::UserContactMissing => GenericError {
            code: "GRA0011",
            message: "Both mobile and email are missing".to_string(),
            status: 500,
        },
        ErrorEnum::PasswordMismatch => GenericError {
            code: "GRA0012",
            message: "Invalid user details".to_string(),
            status: 401,
        },
        ErrorEnum::EmailMobileEmpty => GenericError {
            code: "GRA0013",
            message: "Mobile number and email both cannot be empty".to_string(),
            status: 400,
        },
        ErrorEnum::InvalidOTP => GenericError {
            code: "GRA0014",
            message: "Invalid OTP".to_string(),
            status: 400,
        },
        ErrorEnum::TokenExpired => GenericError {
            code: "GRA0015",
            message: "Auth token is expired".to_string(),
            status: 401,
        },
        ErrorEnum::OAuthProviderNotFound => GenericError {
            code: "GRA0016",
            message: "OAuth invalid provider".to_string(),
            status: 400,
        },
        ErrorEnum::OAuthFailed(e) => GenericError {
            code: "GRA0017",
            message: format!("OAuth failed: {}", e),
            status: 500,
        },
        ErrorEnum::NotYetImplemented => GenericError {
            code: "GRA9999",
            message: "Feature not yet implemented".to_string(),
            status: 500,
        },
    }
}

#[derive(Debug)]
pub enum Error {
    ValidationErrors(ValidationErrors),
    MongoError(mongodb::error::Error),
    MongoOIDError(mongodb::bson::oid::Error),
    RedisError(RedisError),
    JsonError(JsonError),
    BcryptError(bcrypt::BcryptError),
    SystemTimeError(SystemTimeError),
    JWTError(jsonwebtoken::errors::Error),
    ToStrError(ToStrError),
    WebResponseErrorCustom(WebResponseErrorCustom),
    DefinedError(ErrorEnum),
    ParseError(ParseError),
    VarError(VarError),
    ReqwestError(reqwest::Error),
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
            | Error::ParseError(_)
            | Error::VarError(_)
            | Error::ReqwestError(_)
            | Error::JsonError(_) => (json!({ "error": "Internal server error" }), 500),

            Error::JWTError(_) => {
                let error = get_error(&ErrorEnum::UnAuthorized);
                (serde_json::to_value(&error).unwrap(), error.status)
            }

            Error::ValidationErrors(e) => {
                let error = get_error(&ErrorEnum::ValidationError(e.to_string()));
                (serde_json::to_value(&error).unwrap(), error.status)
            }

            Error::WebResponseErrorCustom(e) => (json!({ "error": e.msg }), e.status),
            Error::ToStrError(e) => (json!({"error": e.to_string()}), 400),
            Error::MongoOIDError(_) => (json!({"error": "Invalid bson parameter"}), 400),
            Error::DefinedError(e) => {
                let error = get_error(e);
                (serde_json::to_value(&error).unwrap(), error.status)
            }
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

impl From<mongodb::bson::oid::Error> for Error {
    fn from(value: mongodb::bson::oid::Error) -> Self {
        Error::MongoOIDError(value)
    }
}

impl From<ErrorEnum> for Error {
    fn from(v: ErrorEnum) -> Self {
        Self::DefinedError(v)
    }
}

impl From<ParseError> for Error {
    fn from(value: ParseError) -> Self {
        Error::ParseError(value)
    }
}

impl From<VarError> for Error {
    fn from(value: VarError) -> Self {
        Error::VarError(value)
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::ReqwestError(value)
    }
}
