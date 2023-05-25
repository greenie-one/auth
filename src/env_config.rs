use lazy_static::lazy_static;

lazy_static! {
    pub static ref APP_ENV: String = std::env::var("APP_ENV").expect("APP_ENV should be defined");
}

pub fn load_env() {
    println!("APP_ENV: {:?}", APP_ENV.as_str());
    dotenv::from_filename(format!("./.env.{}", APP_ENV.as_str())).unwrap();
}
