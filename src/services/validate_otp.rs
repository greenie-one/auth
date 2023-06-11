use rand::{thread_rng, Rng};

use crate::{
    database::{mongo::UserModel, redis::REDIS_INSTANCE},
    env_config::APP_ENV,
    error::{Error, ErrorEnum},
};

pub async fn request_otp(user: UserModel) -> Result<(), Error> {
    let contact = if user.mobile_number.is_some() {
        user.mobile_number
    } else if user.email.is_some() {
        user.email
    } else {
        None
    };

    if contact.is_none() {
        return Err(ErrorEnum::UserContactMissing.into());
    }

    let otp = format!("{:06}", thread_rng().gen_range(0..999999));

    REDIS_INSTANCE
        .lock()
        .unwrap()
        .set_ex(format!("{}_otp", contact.unwrap()), 5 * 60, otp)?;

    Ok(())
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
