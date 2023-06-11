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

use crate::{
    database::redis::Redis,
    structs::{GenericError, WebResponseErrorCustom},
};

#[derive(Debug)]
pub enum ErrorEnum {
    UnAuthorized,
    SessionNonExistent,
    UserAlreadyExists,
    InvalidValidationId,
    InvalidRefreshToken,
    LinkedinTokenUnauthenticated,
    LinkedinAuthFailed,
    UserNotFound,
    ProfileNotFound,
    ProfileAlreadyExists,
    UserContactMissing,
    PasswordMismatch,
    EmailMobileEmpty,
    InvalidOTP,
}

fn get_error<'a>(val: &ErrorEnum) -> GenericError<'a> {
    match val {
        ErrorEnum::UnAuthorized => GenericError {
            message: "Unauthorized",
            status: 400,
            code: "GRA0001",
        },
        ErrorEnum::SessionNonExistent => GenericError {
            code: "GRA0002",
            message: "Session does not exist",
            status: 400,
        },
        ErrorEnum::UserAlreadyExists => GenericError {
            code: "GRA0003",
            message: "User already exists",
            status: 409,
        },
        ErrorEnum::InvalidValidationId => GenericError {
            code: "GRA0004",
            message: "Invalid validation ID",
            status: 400,
        },
        ErrorEnum::InvalidRefreshToken => GenericError {
            code: "GRA0005",
            message: "Invalid refresh token",
            status: 400,
        },
        ErrorEnum::LinkedinTokenUnauthenticated => GenericError {
            code: "GRA0006",
            message: "Failed to verify authenticity of token",
            status: 401,
        },
        ErrorEnum::LinkedinAuthFailed => GenericError {
            code: "GRA0007",
            message: "LinkedIn auth failed, %s: %s",
            status: 401,
        },
        ErrorEnum::UserNotFound => GenericError {
            code: "GRA0008",
            message: "User not found",
            status: 404,
        },
        ErrorEnum::ProfileNotFound => GenericError {
            code: "GRA0009",
            message: "Profile not found",
            status: 404,
        },
        ErrorEnum::ProfileAlreadyExists => GenericError {
            code: "GRA0010",
            message: "Profile already exists",
            status: 400,
        },
        ErrorEnum::UserContactMissing => GenericError {
            code: "GRA0011",
            message: "Both mobile and email are missing",
            status: 500,
        },
        ErrorEnum::PasswordMismatch => GenericError {
            code: "GRA0012",
            message: "Invalid user details",
            status: 401,
        },
        ErrorEnum::EmailMobileEmpty => GenericError {
            code: "GRA0013",
            message: "Mobile number and email both cannot be empty",
            status: 400,
        },
        ErrorEnum::InvalidOTP => GenericError {
            code: "GRA0014",
            message: "Invalid OTP",
            status: 400,
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
