use crate::{
    database::{mongo::UserModel, redis::REDIS_INSTANCE},
    dtos::change_password_dto::ValidateForgotPasswordDto,
    error::{Error, ErrorEnum},
    remote::otp::{send_otp, ContactType},
    structs::{ChangePasswordValidationData, ValidationData},
};

use super::signup::ValidationType;

pub async fn request_forgot_pass_otp(
    validation_id: String,
    user_id: String,
    email: String,
) -> Result<(), Error> {
    let otp_resp = send_otp(email, ContactType::EMAIL).await?;
    let otp = otp_resp.get("otp");
    if otp.is_some() {
        let otp = otp.unwrap().to_string();
        REDIS_INSTANCE.lock()?.set_ex(
            format!("change_password_{}", validation_id),
            15 * 60,
            serde_json::to_string(&ChangePasswordValidationData {
                user_id,
                otp: otp.clone(),
            })?,
        )?;
    }

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

    if contact_type == ContactType::MOBILE
        || (contact_type == ContactType::EMAIL && require_email_otp)
    {
        let otp_resp = send_otp(contact.clone(), contact_type).await?;
        let otp = otp_resp.get("otp");
        if otp.is_some() {
            let otp = otp.unwrap().as_str().unwrap();
            REDIS_INSTANCE.lock().unwrap().set_ex(
                format!("{}_otp", contact),
                5 * 60,
                otp.to_string(),
            )?;
        }
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

fn should_validate_otp(validation_type: ValidationType, user: UserModel) -> bool {
    if validation_type == ValidationType::Signup {
        return true;
    }

    user.mobile_number.is_some()
}

pub fn validate_otp(
    user: UserModel,
    otp: String,
    validation_type: ValidationType,
) -> Result<(), Error> {
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

    let otp_key = format!("{}_otp", contact.unwrap());
    let otp_fetched = REDIS_INSTANCE.lock().unwrap().get(otp_key.to_owned())?;

    // Check OTP only on mobile number. Let email pass without OTP
    if should_validate_otp(validation_type, user.clone()) {
        if otp_fetched.is_none() {
            return Err(ErrorEnum::OTPExpired.into());
        }

        if !otp.eq(&otp_fetched.unwrap()) {
            return Err(ErrorEnum::InvalidOTP.into());
        }
    }

    REDIS_INSTANCE.lock().unwrap().del(otp_key)?;
    Ok(())
}

pub async fn generate_and_resend_otp(validation_id: String) -> Result<(), Error> {
    let validation_key = format!("validation_{}", validation_id);
    let validation_data: ValidationData = REDIS_INSTANCE
        .lock()
        .unwrap()
        .get_json(validation_key.to_owned())?;

    let user = validation_data.user;
    let validation_type = validation_data.validation_type;

    let resp = request_login_otp(user, validation_type == ValidationType::Signup).await?;
    println!("Send OTP resp {:?}", resp);

    Ok(())
}
