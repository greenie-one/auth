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

async fn signup(item: web::types::Json<CreateUserDto>) -> Result<HttpResponse, Error> {
    item.validate()?;

    let validation_id = create_temp_user(item.into_inner().clone(), ValidationType::Signup).await?;

    Ok(HttpResponse::build(StatusCode::OK).json(&json!({ "validationId": validation_id })))
}

async fn login(item: web::types::Json<CreateUserDto>) -> Result<HttpResponse, Error> {
    item.validate()?;

    let validation_id = create_temp_user(item.into_inner().clone(), ValidationType::Login).await?;

    Ok(HttpResponse::build(StatusCode::OK).json(&json!({ "validationId": validation_id })))
}

async fn validate_otp(item: web::types::Json<ValidateOtpDto>) -> Result<HttpResponse, Error> {
    item.validate()?;

    let data = validate_by_validation_id(item.into_inner().clone()).await?;

    Ok(HttpResponse::build(StatusCode::OK).json(&data))
}

async fn validate_forgot_password_otp(
    item: web::types::Json<ValidateForgotPasswordDto>,
) -> Result<HttpResponse, Error> {
    item.validate()?;

    validate_change_password(item.into_inner()).await?;
    Ok(HttpResponse::build(StatusCode::OK).finish())
}

async fn forgot_password(item: web::types::Json<ForgotPasswordDto>) -> Result<HttpResponse, Error> {
    item.validate()?;

    let data = initiate_forgot_password(item.email.clone()).await?;
    Ok(HttpResponse::build(StatusCode::OK).json(&json!({ "validationId": data })))
}

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
                change_password_service(c.sub, item.into_inner(), false).await?;
                resp.status(StatusCode::OK)
            }
            Err(_) => resp.status(StatusCode::UNAUTHORIZED),
        };
    }

    Ok(resp.finish())
}

fn get_route(route: &str) -> String {
    let app_env = std::env::var("APP_ENV").expect("APP_ENV should be defined");
    if app_env == "local" {
        format!("/auth{}", route)
    } else {
        route.to_owned()
    }
}

#[ntex::main]
async fn main() -> io::Result<()> {
    env_config::load_env();
    env::set_var("RUST_LOG", "ntex=info");
    env_logger::init();

    web::server(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .route(get_route("/signup").as_str(), web::post().to(signup))
            .route(
                get_route("/validate_token").as_str(),
                web::get().to(validate_token),
            )
            .route(get_route("/login").as_str(), web::post().to(login))
            .route(
                get_route("/validateOTP").as_str(),
                web::post().to(validate_otp),
            )
            .route(
                get_route("/validate_forgot_password").as_str(),
                web::post().to(validate_forgot_password_otp),
            )
            .route(
                get_route("/forgot_password").as_str(),
                web::post().to(forgot_password),
            )
            .route(
                get_route("/change_password").as_str(),
                web::post().to(change_password),
            )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
