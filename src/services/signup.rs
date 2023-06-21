use std::fmt;

use crate::{
    database::{
        mongo::{UserModel, MONGO_DB_INSTANCE},
        redis::REDIS_INSTANCE,
    },
    dtos::{signup_dto::CreateUserDto, validate_otp_dto::ValidateOtpDto},
    error::{Error, ErrorEnum},
    structs::{AccessTokenResponse, ValidationData},
};

use mongodb::bson::oid::ObjectId;
use ntex::rt::spawn;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    token::create_token,
    validate_otp::{request_login_otp, validate_otp},
};

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug, Clone, Copy)]
pub enum ValidationType {
    Login,
    Signup,
}

impl fmt::Display for ValidationType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValidationType::Login => write!(f, "LOGIN"),
            ValidationType::Signup => write!(f, "SIGNUP"),
        }
    }
}

fn parse_user(data: CreateUserDto) -> Result<UserModel, Error> {
    let hashed_password = if data.password.is_some() {
        Some(bcrypt::hash(data.password.unwrap(), bcrypt::DEFAULT_COST)?)
    } else {
        None
    };

    Ok(UserModel {
        _id: Some(ObjectId::new()),
        email: data.email,
        mobile_number: data.mobile_number,
        password: hashed_password,
        roles: vec!["default".to_string()],
    })
}

fn parse_and_validate_user(
    data: CreateUserDto,
    existing_user: UserModel,
) -> Result<UserModel, Error> {
    let mut verify: bool = false;
    if data.email.is_some() {
        verify = bcrypt::verify(
            data.password.unwrap(),
            existing_user.clone().password.unwrap().as_str(),
        )?;
    }

    if data.mobile_number.is_some() {
        verify = true;
    }

    if verify {
        return Ok(existing_user);
    }

    Err(ErrorEnum::PasswordMismatch.into())
}

pub async fn create_temp_user(
    data: CreateUserDto,
    validation_type: ValidationType,
) -> Result<String, Error> {
    if data.email.is_none() && data.mobile_number.is_none() {
        return Err(ErrorEnum::EmailMobileEmpty.into());
    }

    let database = MONGO_DB_INSTANCE.get().await;

    let user = database
        .find_user(data.email.clone(), data.mobile_number.clone(), None)
        .await?;

    let parsed_user: UserModel = match validation_type {
        ValidationType::Login => {
            if user.is_none() {
                return Err(ErrorEnum::UserNotFound.into());
            }

            parse_and_validate_user(data, user.unwrap())
        }
        ValidationType::Signup => {
            if user.is_some() {
                return Err(ErrorEnum::UserAlreadyExists(user.unwrap()).into());
            }

            parse_user(data)
        }
    }?;

    let validation_id = Uuid::new_v4().to_string();

    let validation_data = ValidationData {
        validation_type,
        user: parsed_user.clone(),
    };

    REDIS_INSTANCE.lock().unwrap().set_ex(
        format!("validation_{}", validation_id),
        15 * 60,
        serde_json::to_string(&validation_data)?.to_string(),
    )?;

    spawn(async move {
        let resp = request_login_otp(parsed_user, validation_type == ValidationType::Signup).await;
        println!("Send OTP resp {:?}", resp);
    });

    Ok(validation_id)
}

pub async fn insert_user(mut user: UserModel) -> Result<UserModel, Error> {
    let mongodb = MONGO_DB_INSTANCE.get().await;

    let existing = mongodb
        .find_user(user.clone().email, user.clone().mobile_number, None)
        .await?;
    if existing.is_some() {
        return Err(ErrorEnum::UserAlreadyExists(existing.unwrap()).into());
    }

    let _id = mongodb.create_user(user.clone()).await?;

    user._id = _id.inserted_id.as_object_id();

    Ok(user)
}

pub async fn validate_by_validation_id(data: ValidateOtpDto) -> Result<AccessTokenResponse, Error> {
    let validation_key = format!("validation_{}", data.validation_id);
    let validation_data: Result<ValidationData, Error> = REDIS_INSTANCE
        .lock()
        .unwrap()
        .get_json(validation_key.to_owned());

    match validation_data {
        Ok(mut d) => {
            validate_otp(d.user.clone(), data.otp)?;

            REDIS_INSTANCE.lock().unwrap().del(validation_key)?;

            if d.validation_type.eq(&ValidationType::Signup) {
                d.user = insert_user(d.user).await?;
            }

            create_token(d.user)
        }
        Err(_) => Err(ErrorEnum::InvalidValidationId.into()),
    }
}
