pub fn load_env() {
    let app_env = std::env::var("APP_ENV").expect("APP_ENV should be defined");
    dotenv::from_filename(format!(".env.{}", app_env)).unwrap();
}
