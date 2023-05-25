use std::fmt;

use crate::{
    database::{
        mongo::{UserModel, MONGO_DB_INSTANCE},
        redis::REDIS_INSTANCE,
    },
    dtos::{signup_dto::CreateUserDto, validate_otp_dto::ValidateOtpDto},
    error::Error,
    structs::{AccessTokenResponse, ValidationData},
};

use mongodb::bson::oid::ObjectId;
use ntex::rt::spawn;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    token::create_token,
    validate_otp::{request_otp, validate_otp},
};

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug)]
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

fn create_user(data: CreateUserDto) -> Result<UserModel, Error> {
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

fn validate_user(data: CreateUserDto, existing_user: UserModel) -> Result<UserModel, Error> {
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

    Err(Error::new("Invalid username or password", 401))
}

pub async fn create_temp_user(
    data: CreateUserDto,
    validation_type: ValidationType,
) -> Result<String, Error> {
    if data.email.is_none() && data.mobile_number.is_none() {
        return Err(Error::new(
            "Mobile number and email both cannot be empty",
            400,
        ));
    }

    let database = MONGO_DB_INSTANCE.get().await;

    let user = database
        .find_user(data.email.clone(), data.mobile_number.clone(), None)
        .await?;

    let parsed_user: UserModel = match validation_type {
        ValidationType::Login => {
            if user.is_none() {
                return Err(Error::new("User does not exist", 400));
            }

            validate_user(data, user.unwrap())
        }
        ValidationType::Signup => {
            if user.is_some() {
                return Err(Error::new("User already exists", 400));
            }

            create_user(data)
        }
    }?;

    let validation_id = Uuid::new_v4().to_string();
    let validation_type = validation_type;

    let validation_data = ValidationData {
        validation_type,
        user: parsed_user.clone(),
    };

    REDIS_INSTANCE.lock().unwrap().set_ex(
        format!("validation_{}", validation_id),
        15 * 60,
        serde_json::to_string(&validation_data)?.to_string(),
    )?;

    spawn(async { request_otp(parsed_user).await });

    Ok(validation_id)
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
                let _id = MONGO_DB_INSTANCE
                    .get()
                    .await
                    .create_user(d.user.clone())
                    .await?;
                d.user._id = _id.inserted_id.as_object_id();
            }

            create_token(d.user)
        }
        Err(_) => Err(Error::new("Invalid validation ID", 400)),
    }
}
