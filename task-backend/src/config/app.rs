use std::env;

#[derive(Clone, Debug)]
pub struct SecurityConfig {
    pub cookie_secure: bool,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct AppConfig {
    pub environment: String,
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub frontend_url: String,
    pub security: SecurityConfig,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, String> {
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        let is_production = environment == "production";

        Ok(Self {
            environment,
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "5000".to_string())
                .parse()
                .map_err(|_| "Invalid PORT value")?,
            database_url: env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set")?,
            jwt_secret: env::var("JWT_SECRET")
                .or_else(|_| env::var("JWT_SECRET_KEY"))
                .map_err(|_| "JWT_SECRET or JWT_SECRET_KEY must be set")?,
            frontend_url: env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            security: SecurityConfig {
                cookie_secure: is_production,
            },
        })
    }

    pub fn is_development(&self) -> bool {
        self.environment == "development"
    }

    pub fn is_test(&self) -> bool {
        self.environment == "test"
    }

    pub fn is_production(&self) -> bool {
        self.environment == "production"
    }

    /// テスト用の設定を作成
    #[allow(dead_code)]
    pub fn for_testing() -> Self {
        // 環境変数から読み込み、なければデフォルト値を使用
        Self {
            environment: env::var("ENVIRONMENT").unwrap_or_else(|_| "test".to_string()),
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "5000".to_string())
                .parse()
                .unwrap_or(5000),
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgresql://postgres:postgres@localhost:5432/test_db".to_string()
            }),
            jwt_secret: env::var("JWT_SECRET")
                .or_else(|_| env::var("JWT_SECRET_KEY"))
                .unwrap_or_else(|_| {
                    "test-secret-key-that-is-at-least-32-characters-long".to_string()
                }),
            frontend_url: env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            security: SecurityConfig {
                cookie_secure: false,
            },
        }
    }
}

// Backward compatibility
pub type Config = AppConfig;
