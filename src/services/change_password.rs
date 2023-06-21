use uuid::Uuid;

use crate::{
    database::{mongo::MONGO_DB_INSTANCE, redis::REDIS_INSTANCE},
    dtos::change_password_dto::{ChangePasswordDto, ValidateForgotPasswordDto},
    error::{Error, ErrorEnum},
    structs::ChangePasswordValidationData,
    validate_forgot_password_otp,
};

use super::validate_otp::{request_forgot_pass_otp, validate_forgot_pass_otp};

pub async fn initiate_forgot_password(email: String) -> Result<String, Error> {
    let mongodb = MONGO_DB_INSTANCE.get().await;

    let user = mongodb.find_user(Some(email.clone()), None, None).await?;
    if user.is_none() {
        return Err(ErrorEnum::UserNotFound.into());
    }

    let user = user.unwrap();

    let validation_id = Uuid::new_v4().to_string();
    request_forgot_pass_otp(validation_id.clone(), user._id.unwrap().to_string(), email).await?;

    Ok(validation_id)
}

pub async fn validate_change_password(data: ValidateForgotPasswordDto) -> Result<(), Error> {
    let validation_data = validate_forgot_pass_otp(data.clone())?;
    change_password(validation_data.user_id, data.into(), true).await?;
    Ok(())
}

pub async fn change_password(
    user_id: String,
    data: ChangePasswordDto,
    bypass_pass_check: bool,
) -> Result<(), Error> {
    let mongodb = MONGO_DB_INSTANCE.get().await;
    let user = mongodb.find_user(None, None, Some(user_id.clone())).await?;

    if user.is_none() {
        return Err(ErrorEnum::UserNotFound.into());
    }

    let user = user.unwrap();

    let is_password_correct = bypass_pass_check
        || bcrypt::verify(data.current_password.unwrap(), &user.password.unwrap())?;
    if is_password_correct {
        mongodb
            .update_password(
                user_id,
                bcrypt::hash(data.new_password, bcrypt::DEFAULT_COST)?,
            )
            .await?;

        return Ok(());
    }

    Err(ErrorEnum::PasswordMismatch.into())
}
