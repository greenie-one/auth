use std::{env, io};

use ntex::http::StatusCode;
use ntex::web::{self, middleware, App, HttpRequest, HttpResponse};
use serde_json::json;
use validator::Validate;

use crate::dtos::signup_dto::CreateUserDto;
use crate::dtos::validate_otp_dto::ValidateOtpDto;
use crate::error::Error;
use crate::services::signup::{create_temp_user, validate_by_validation_id, ValidationType};

mod database;
mod dtos;
mod env_config;
mod error;
mod services;
mod structs;

#[web::post("/validate_token")]
async fn validate_token(req: HttpRequest) -> Result<HttpResponse, Error> {
    println!("REQ: {:?}", req);
    Ok(HttpResponse::build(StatusCode::OK).finish())
}

#[web::post("/signup")]
async fn signup(item: web::types::Json<CreateUserDto>) -> Result<HttpResponse, Error> {
    item.validate()?;

    let validation_id = create_temp_user(item.into_inner().clone(), ValidationType::Signup).await?;

    Ok(HttpResponse::build(StatusCode::OK).json(&json!({ "validationId": validation_id })))
}

#[web::post("/login")]
async fn login(item: web::types::Json<CreateUserDto>) -> Result<HttpResponse, Error> {
    item.validate()?;

    let validation_id = create_temp_user(item.into_inner().clone(), ValidationType::Login).await?;

    Ok(HttpResponse::build(StatusCode::OK).json(&json!({ "validationId": validation_id })))
}

#[web::post("/validateOTP")]
async fn validate_otp(item: web::types::Json<ValidateOtpDto>) -> Result<HttpResponse, Error> {
    item.validate()?;

    let data = validate_by_validation_id(item.into_inner().clone()).await?;

    Ok(HttpResponse::build(StatusCode::OK).json(&data))
}

#[ntex::main]
async fn main() -> io::Result<()> {
    env_config::load_env();
    env::set_var("RUST_LOG", "ntex=info");
    env_logger::init();

    web::server(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(signup)
            .service(login)
            .service(validate_otp)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
