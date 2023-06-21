use rand::{thread_rng, Rng};

use crate::{
    database::{mongo::UserModel, redis::REDIS_INSTANCE},
    dtos::change_password_dto::ValidateForgotPasswordDto,
    env_config::APP_ENV,
    error::{Error, ErrorEnum},
    remote::otp::{send_otp, ContactType},
    structs::ChangePasswordValidationData,
};

pub async fn request_forgot_pass_otp(
    validation_id: String,
    user_id: String,
    email: String,
) -> Result<(), Error> {
    let otp = format!("{:06}", thread_rng().gen_range(0..999999));

    REDIS_INSTANCE.lock()?.set_ex(
        format!("change_password_{}", validation_id),
        15 * 60,
        serde_json::to_string(&ChangePasswordValidationData {
            user_id,
            otp: otp.clone(),
        })?,
    )?;

    send_otp(email, otp, ContactType::EMAIL).await?;
    Ok(())
}

pub async fn request_login_otp(user: UserModel, require_email_otp: bool) -> Result<(), Error> {
    let (contact, contact_type) = if user.mobile_number.is_some() {
        (user.mobile_number, ContactType::MOBILE)
    } else if user.email.is_some() {
        (user.email, ContactType::EMAIL)
    } else {
        return Err(ErrorEnum::UserContactMissing.into());
    };

    let contact = contact.unwrap();

    let otp = format!("{:06}", thread_rng().gen_range(0..999999));

    REDIS_INSTANCE
        .lock()
        .unwrap()
        .set_ex(format!("{}_otp", contact), 5 * 60, otp.clone())?;

    if contact_type == ContactType::MOBILE
        || (contact_type == ContactType::EMAIL && require_email_otp)
    {
        send_otp(contact, otp, contact_type).await?;
    }

    Ok(())
}

pub fn validate_forgot_pass_otp(
    data: ValidateForgotPasswordDto,
) -> Result<ChangePasswordValidationData, Error> {
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
                return Err(ErrorEnum::InvalidOTP.into());
            }

            REDIS_INSTANCE
                .lock()?
                .del(format!("change_password_{}", validation_id))?;

            Ok(validation_data)
        }
        Err(e) => {
            println!("{:?}", e);
            Err(ErrorEnum::InvalidValidationId.into())
        }
    }
}

pub fn validate_otp(user: UserModel, otp: String) -> Result<(), Error> {
    let contact = if user.mobile_number.is_some() {
        user.mobile_number.clone()
    } else if user.email.is_some() {
        user.email.clone()
    } else {
        None
    };

    if contact.is_none() {
        return Err(ErrorEnum::UserContactMissing.into());
    }

    if APP_ENV.as_str() != "production" && otp == "123456" {
        return Ok(());
    }

    let otp_key = format!("{}_otp", contact.unwrap());
    let otp_fetched: String = REDIS_INSTANCE.lock().unwrap().get(otp_key.to_owned())?;

    // Check OTP only on mobile number. Let email pass without OTP
    if (user.mobile_number.is_some() && otp.eq(&otp_fetched)) || (user.email.is_some()) {
        REDIS_INSTANCE.lock().unwrap().del(otp_key)?;
        return Ok(());
    }

    Err(ErrorEnum::InvalidOTP.into())
}
