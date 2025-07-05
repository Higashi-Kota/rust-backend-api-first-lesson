use std::env;

#[derive(Clone, Debug)]
pub struct SecurityConfig {
    pub cookie_secure: bool,
}

#[derive(Clone, Debug)]
pub struct ServerConfig {
    #[allow(dead_code)]
    pub body_limit: usize,
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub environment: String,
    pub host: String,
    pub port: u16,
    #[allow(dead_code)]
    pub cors_allowed_origins: Vec<String>,
    pub database_url: String,
    #[allow(dead_code)]
    pub jwt_secret: String,
    #[allow(dead_code)]
    pub frontend_url: String,
    pub security: SecurityConfig,
    #[allow(dead_code)]
    pub server: ServerConfig,
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
            cors_allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3001".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            database_url: env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set")?,
            jwt_secret: env::var("JWT_SECRET")
                .or_else(|_| env::var("JWT_SECRET_KEY"))
                .map_err(|_| "JWT_SECRET or JWT_SECRET_KEY must be set")?,
            frontend_url: env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:3001".to_string()),
            security: SecurityConfig {
                cookie_secure: is_production,
            },
            server: ServerConfig {
                body_limit: 50 * 1024 * 1024, // 50MB
            },
        })
    }

    pub fn is_development(&self) -> bool {
        self.environment == "development"
    }

    #[allow(dead_code)]
    pub fn is_test(&self) -> bool {
        self.environment == "test"
    }

    #[allow(dead_code)]
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
            cors_allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3001".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgresql://postgres:postgres@localhost:5432/test_db".to_string()
            }),
            jwt_secret: env::var("JWT_SECRET")
                .or_else(|_| env::var("JWT_SECRET_KEY"))
                .unwrap_or_else(|_| {
                    "test-secret-key-that-is-at-least-32-characters-long".to_string()
                }),
            frontend_url: env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:3001".to_string()),
            security: SecurityConfig {
                cookie_secure: false,
            },
            server: ServerConfig {
                body_limit: 50 * 1024 * 1024,
            },
        }
    }
}

// Backward compatibility
pub type Config = AppConfig;
