// src/config.rs
use dotenvy::dotenv;
use std::env;
use std::fmt;

use crate::utils::email::EmailConfig;
use crate::utils::jwt::JwtConfig;
use crate::utils::password::{Argon2Config, PasswordPolicy};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub email: EmailConfig,
    pub password: PasswordConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub addr: String,
    pub environment: Environment,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub schema: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PasswordConfig {
    pub policy: PasswordPolicy,
    pub argon2: Argon2Config,
}

#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub cookie_secure: bool,
    #[allow(dead_code)]
    pub development_mode: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Development,
    Production,
    Test,
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Environment::Development => write!(f, "development"),
            Environment::Production => write!(f, "production"),
            Environment::Test => write!(f, "test"),
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    EnvVar(String),
    #[allow(dead_code)]
    Parse(String),
    Validation(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::EnvVar(msg) => write!(f, "Environment variable error: {}", msg),
            ConfigError::Parse(msg) => write!(f, "Parse error: {}", msg),
            ConfigError::Validation(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<env::VarError> for ConfigError {
    fn from(err: env::VarError) -> Self {
        ConfigError::EnvVar(err.to_string())
    }
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        let environment = Environment::from_env()?;

        let server = ServerConfig::from_env(&environment)?;
        let database = DatabaseConfig::from_env()?;
        let jwt = JwtConfig::from_env().map_err(|e| ConfigError::Validation(e.to_string()))?;
        let email = EmailConfig::default();
        let password = PasswordConfig::from_env()?;
        let security = SecurityConfig::from_env(&environment)?;

        let config = AppConfig {
            server,
            database,
            jwt,
            email,
            password,
            security,
        };

        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        self.jwt
            .validate()
            .map_err(|e| ConfigError::Validation(e.to_string()))?;
        self.email
            .validate()
            .map_err(|e| ConfigError::Validation(e.to_string()))?;
        Ok(())
    }

    pub fn is_development(&self) -> bool {
        self.server.environment == Environment::Development
    }
}

impl Environment {
    fn from_env() -> Result<Self, ConfigError> {
        let env_str = env::var("APP_ENV")
            .or_else(|_| env::var("RUST_ENV"))
            .unwrap_or_else(|_| {
                if env::var("RUST_TEST").is_ok() {
                    "test".to_string()
                } else {
                    "development".to_string()
                }
            });

        match env_str.to_lowercase().as_str() {
            "development" | "dev" => Ok(Environment::Development),
            "production" | "prod" => Ok(Environment::Production),
            "test" => Ok(Environment::Test),
            _ => Ok(Environment::Development),
        }
    }
}

impl ServerConfig {
    fn from_env(environment: &Environment) -> Result<Self, ConfigError> {
        let addr = env::var("SERVER_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

        Ok(ServerConfig {
            addr,
            environment: environment.clone(),
        })
    }
}

impl DatabaseConfig {
    fn from_env() -> Result<Self, ConfigError> {
        let url = env::var("DATABASE_URL")?;
        let schema = env::var("DB_SCHEMA").ok();

        Ok(DatabaseConfig { url, schema })
    }
}

impl PasswordConfig {
    fn from_env() -> Result<Self, ConfigError> {
        let policy = PasswordPolicy::from_env();
        let argon2 = Argon2Config::from_env();

        Ok(PasswordConfig { policy, argon2 })
    }
}

impl SecurityConfig {
    fn from_env(environment: &Environment) -> Result<Self, ConfigError> {
        let development_mode = matches!(environment, Environment::Development | Environment::Test);
        let cookie_secure = !development_mode;

        Ok(SecurityConfig {
            cookie_secure,
            development_mode,
        })
    }
}

// Minimal Config struct for backward compatibility with database operations
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
}

impl Config {
    pub fn from_app_config(app_config: &AppConfig) -> Self {
        Config {
            database_url: app_config.database.url.clone(),
        }
    }
}

impl AppConfig {
    /// Test configuration
    #[allow(dead_code)]
    pub fn for_testing() -> Self {
        use crate::utils::email::EmailConfig;
        use crate::utils::jwt::JwtConfig;
        use crate::utils::password::{Argon2Config, PasswordPolicy};

        let environment = Environment::Test;

        let server = ServerConfig {
            addr: "127.0.0.1:0".to_string(),
            environment: environment.clone(),
        };

        let database = DatabaseConfig {
            url: "postgres://postgres:password@localhost:5432/taskdb_test".to_string(),
            schema: None,
        };

        let jwt = JwtConfig {
            secret_key: "test_secret_key_must_be_at_least_32_characters_long_for_testing"
                .to_string(),
            access_token_expiry_minutes: 15,
            refresh_token_expiry_days: 7,
            issuer: "test-task-backend".to_string(),
            audience: "test-users".to_string(),
        };

        let email = EmailConfig {
            provider: crate::utils::email::EmailProvider::Development,
            smtp_host: "localhost".to_string(),
            smtp_port: 1025,
            from_email: "test@example.com".to_string(),
            from_name: "Test Backend".to_string(),
            mailgun_api_key: None,
            mailgun_domain: None,
            development_mode: true,
        };

        let password = PasswordConfig {
            policy: PasswordPolicy::default(),
            argon2: Argon2Config::default(),
        };

        let security = SecurityConfig {
            cookie_secure: false,
            development_mode: true,
        };

        AppConfig {
            server,
            database,
            jwt,
            email,
            password,
            security,
        }
    }
}
