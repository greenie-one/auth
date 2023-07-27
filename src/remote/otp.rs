use serde_json::json;

use crate::error::Error;

#[derive(PartialEq)]
pub enum ContactType {
    EMAIL,
    MOBILE,
}

fn get_otp_type(contact_type: ContactType) -> String {
    match contact_type {
        ContactType::EMAIL => "EMAIL".to_owned(),
        ContactType::MOBILE => "MOBILE".to_owned(),
    }
}

pub async fn send_otp(
    contact: String,
    contact_type: ContactType,
) -> Result<serde_json::Value, Error> {
    println!("Sending OTP");

    let client = reqwest::ClientBuilder::new().build()?;

    let resp: serde_json::Value = client
        .post(format!(
            "{}/otp/send",
            std::env::var("REMOTE_BASE_URL").unwrap()
        ))
        .json(&json!({ "type": get_otp_type(contact_type), "contact": contact}))
        .send()
        .await?
        .json()
        .await?;

    println!("Got OTP response {:?}", resp);

    Ok(resp)
}
