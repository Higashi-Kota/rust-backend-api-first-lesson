// src/config.rs
use dotenvy::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_addr: String,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        dotenv().ok(); // .env ファイルを読み込む (存在しなくてもエラーにしない)

        let database_url = env::var("DATABASE_URL")?;
        let server_addr = env::var("SERVER_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

        Ok(Config {
            database_url,
            server_addr,
        })
    }
}
