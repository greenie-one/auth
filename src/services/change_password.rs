use rand::{thread_rng, Rng};
use uuid::Uuid;

use crate::{
    database::{mongo::MONGO_DB_INSTANCE, redis::REDIS_INSTANCE},
    dtos::change_password_dto::{ChangePasswordDto, ValidateForgotPasswordDto},
    error::Error,
    structs::ChangePasswordValidationData,
};

pub async fn initiate_forgot_password(email: String) -> Result<String, Error> {
    let mongodb = MONGO_DB_INSTANCE.get().await;

    let user = mongodb.find_user(Some(email.clone()), None, None).await?;
    if user.is_none() {
        return Err(Error::new("User not found", 400));
    }

    let user = user.unwrap();

    let validation_id = Uuid::new_v4().to_string();
    let otp = format!("{:06}", thread_rng().gen_range(0..999999));

    REDIS_INSTANCE.lock()?.set_ex(
        format!("change_password_{}", validation_id),
        15 * 60,
        serde_json::to_string(&ChangePasswordValidationData {
            otp,
            user_id: user._id.unwrap().to_string(),
        })?,
    )?;

    Ok(validation_id)
}

pub async fn validate_change_password(data: ValidateForgotPasswordDto) -> Result<(), Error> {
    let validation_id = data.validation_id.clone();

    let res = REDIS_INSTANCE
        .lock()?
        .get_json::<ChangePasswordValidationData>(format!(
            "change_password_{}",
            validation_id.clone()
        ));

    match res {
        Ok(validation_data) => {
            if data.otp != validation_data.otp {
                return Err(Error::new("Invalid OTP", 400));
            }

            change_password(validation_data.user_id, data.into(), true).await?;

            REDIS_INSTANCE
                .lock()?
                .del(format!("change_password_{}", validation_id))
        }
        Err(e) => {
            println!("{:?}", e);
            Err(Error::new("Invalid validation ID", 400))
        }
    }
}

pub async fn change_password(
    user_id: String,
    data: ChangePasswordDto,
    bypass_pass_check: bool,
) -> Result<(), Error> {
    let mongodb = MONGO_DB_INSTANCE.get().await;
    let user = mongodb.find_user(None, None, Some(user_id.clone())).await?;

    if user.is_none() {
        return Err(Error::new("User not found", 400));
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

    Err(Error::new("Incorrect old password", 401))
}
