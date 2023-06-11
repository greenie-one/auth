use std::{env, io};

use ntex::http::StatusCode;
use ntex::web::{self, middleware, App, HttpRequest, HttpResponse};
use serde_json::json;
use services::change_password::change_password as change_password_service;
use validator::Validate;

use crate::dtos::change_password_dto::{
    ChangePasswordDto, ForgotPasswordDto, ValidateForgotPasswordDto,
};
use crate::dtos::signup_dto::CreateUserDto;
use crate::dtos::validate_otp_dto::ValidateOtpDto;
use crate::error::Error;
use crate::services::change_password::{initiate_forgot_password, validate_change_password};
use crate::services::signup::{create_temp_user, validate_by_validation_id, ValidationType};
use crate::services::token::decode_token;

mod database;
mod dtos;
mod env_config;
mod error;
mod services;
mod structs;

#[web::get("/validate_token")]
async fn validate_token(req: HttpRequest) -> Result<HttpResponse, Error> {
    let mut resp = HttpResponse::build(StatusCode::OK);
    let auth_token = req.headers().get("authorization");

    if auth_token.is_some() {
        let token_stripped = &auth_token.unwrap().to_str()?[7..];
        let claims = decode_token(token_stripped);
        println!("{:?}", claims);
        match claims {
            Ok(c) => resp.set_header("x-user-details", serde_json::to_string(&c)?),
            Err(_) => resp.status(StatusCode::UNAUTHORIZED),
        };
    }

    Ok(resp.finish())
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

#[web::post("/validate_forgot_password")]
async fn validate_forgot_password_otp(
    item: web::types::Json<ValidateForgotPasswordDto>,
) -> Result<HttpResponse, Error> {
    item.validate()?;

    validate_change_password(item.into_inner()).await?;
    Ok(HttpResponse::build(StatusCode::OK).finish())
}

#[web::post("/forgot_password")]
async fn forgot_password(item: web::types::Json<ForgotPasswordDto>) -> Result<HttpResponse, Error> {
    item.validate()?;

    let data = initiate_forgot_password(item.email.clone()).await?;
    Ok(HttpResponse::build(StatusCode::OK).json(&json!({ "validationId": data })))
}

#[web::post("/change_password")]
async fn change_password(
    req: HttpRequest,
    item: web::types::Json<ChangePasswordDto>,
) -> Result<HttpResponse, Error> {
    let mut resp = HttpResponse::build(StatusCode::OK);
    let auth_token = req.headers().get("authorization");

    if auth_token.is_some() {
        let token_stripped = &auth_token.unwrap().to_str()?[7..];
        let claims = decode_token(token_stripped);
        println!("{:?}", claims);
        match claims {
            Ok(c) => {
                item.validate()?;
                change_password_service(c.sub, item.into_inner()).await?;
                resp.status(StatusCode::OK)
            }
            Err(_) => resp.status(StatusCode::UNAUTHORIZED),
        };
    }

    Ok(resp.finish())
}

#[ntex::main]
async fn main() -> io::Result<()> {
    let app_env = std::env::var("APP_ENV").expect("APP_ENV should be defined");
    println!("APP_ENV: {}", app_env);
    env_config::load_env();
    env::set_var("RUST_LOG", "ntex=info");
    env_logger::init();

    web::server(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(signup)
            .service(login)
            .service(validate_otp)
            .service(validate_token)
            .service(forgot_password)
            .service(validate_forgot_password_otp)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
