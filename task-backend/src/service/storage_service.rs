// task-backend/src/service/storage_service.rs

use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::{Client, Config};
use std::time::Duration;
use uuid::Uuid;

use crate::error::AppResult;
use crate::utils::error_helper::internal_server_error;
use std::sync::Arc;

/// ストレージプロバイダーの種類
#[derive(Debug, Clone, PartialEq)]
pub enum StorageProvider {
    MinIO,
    R2,
}

impl StorageProvider {
    /// 環境変数からプロバイダーを判定
    pub fn from_env() -> Self {
        match std::env::var("STORAGE_PROVIDER")
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "r2" | "cloudflare" | "cloudflare-r2" => Self::R2,
            "minio" => Self::MinIO,
            _ => {
                // APP_ENVに基づくデフォルト選択
                match std::env::var("APP_ENV")
                    .unwrap_or_else(|_| "development".to_string())
                    .to_lowercase()
                    .as_str()
                {
                    "production" | "staging" => Self::R2,
                    _ => Self::MinIO,
                }
            }
        }
    }
}

/// ストレージサービスのトレイト定義
#[async_trait]
pub trait StorageService: Send + Sync {
    /// ファイルをアップロード
    async fn upload(&self, file_data: Vec<u8>, content_type: &str) -> AppResult<String>;

    /// ファイルをダウンロード
    async fn download(&self, key: &str) -> AppResult<Vec<u8>>;

    /// ファイルを削除
    async fn delete(&self, key: &str) -> AppResult<()>;

    /// ファイルの存在確認
    async fn exists(&self, key: &str) -> AppResult<bool>;

    /// 署名付きダウンロードURLを生成
    async fn generate_download_url(&self, key: &str, expires_in_seconds: u64) -> AppResult<String>;

    /// 署名付きアップロードURLを生成
    async fn generate_upload_url(&self, key: &str, expires_in_seconds: u64) -> AppResult<String>;
}

/// S3互換ストレージサービスの実装
pub struct S3StorageService {
    client: Client,
    bucket: String,
}

impl S3StorageService {
    /// 新しいS3ストレージサービスのインスタンスを作成
    pub async fn new(config: StorageConfig) -> AppResult<Self> {
        let credentials = Credentials::new(
            &config.access_key,
            &config.secret_key,
            None,
            None,
            "storage_service",
        );

        // プロバイダーに応じた設定を適用
        let mut s3_config_builder = Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(config.region.clone()))
            .endpoint_url(&config.endpoint)
            .credentials_provider(credentials);

        // プロバイダー固有の設定
        match config.provider {
            StorageProvider::MinIO => {
                // MinIOはpath styleを強制
                s3_config_builder = s3_config_builder.force_path_style(true);
            }
            StorageProvider::R2 => {
                // R2はvirtual-hosted styleをサポート（デフォルト）
                // force_path_style(false)がデフォルトなので設定不要
            }
        }

        let s3_config = s3_config_builder.build();
        let client = Client::from_conf(s3_config);

        Ok(Self {
            client,
            bucket: config.bucket,
        })
    }

    /// 画像ファイル用のストレージキーを生成
    fn generate_image_key() -> String {
        format!("images/{}/{}", Utc::now().format("%Y/%m"), Uuid::new_v4())
    }
}

#[async_trait]
impl StorageService for S3StorageService {
    async fn upload(&self, file_data: Vec<u8>, content_type: &str) -> AppResult<String> {
        let key = Self::generate_image_key();

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(ByteStream::from(file_data))
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| {
                internal_server_error(e, "s3_storage_service::upload", "Failed to upload file")
            })?;

        Ok(key)
    }

    async fn download(&self, key: &str) -> AppResult<Vec<u8>> {
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| crate::error::AppError::NotFound(format!("File not found: {}", e)))?;

        let data = response.body.collect().await.map_err(|e| {
            internal_server_error(
                e,
                "s3_storage_service::download",
                "Failed to read file data",
            )
        })?;

        Ok(data.to_vec())
    }

    async fn delete(&self, key: &str) -> AppResult<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| {
                internal_server_error(e, "s3_storage_service::delete", "Failed to delete file")
            })?;

        Ok(())
    }

    async fn exists(&self, key: &str) -> AppResult<bool> {
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                // オブジェクトが存在しない場合は false を返す
                if e.to_string().contains("NoSuchKey") || e.to_string().contains("404") {
                    Ok(false)
                } else {
                    Err(internal_server_error(
                        e,
                        "s3_storage_service::exists",
                        "Failed to check file existence",
                    ))
                }
            }
        }
    }

    async fn generate_download_url(&self, key: &str, expires_in_seconds: u64) -> AppResult<String> {
        let expires_in = Duration::from_secs(expires_in_seconds);
        let presigning_config = PresigningConfig::expires_in(expires_in).map_err(|e| {
            internal_server_error(
                e,
                "s3_storage_service::generate_download_url",
                "Failed to create presigning config",
            )
        })?;

        let presigned_request = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(presigning_config)
            .await
            .map_err(|e| {
                internal_server_error(
                    e,
                    "s3_storage_service::generate_download_url",
                    "Failed to generate presigned URL",
                )
            })?;

        Ok(presigned_request.uri().to_string())
    }

    async fn generate_upload_url(&self, key: &str, expires_in_seconds: u64) -> AppResult<String> {
        let expires_in = Duration::from_secs(expires_in_seconds);
        let presigning_config = PresigningConfig::expires_in(expires_in).map_err(|e| {
            internal_server_error(
                e,
                "s3_storage_service::generate_upload_url",
                "Failed to create presigning config",
            )
        })?;

        let presigned_request = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(presigning_config)
            .await
            .map_err(|e| {
                internal_server_error(
                    e,
                    "s3_storage_service::generate_upload_url",
                    "Failed to generate presigned upload URL",
                )
            })?;

        Ok(presigned_request.uri().to_string())
    }
}

/// ストレージ設定
#[derive(Clone)]
pub struct StorageConfig {
    pub provider: StorageProvider,
    pub endpoint: String,
    pub bucket: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
}

impl StorageConfig {
    /// 環境変数から設定を読み込み
    pub fn from_env() -> AppResult<Self> {
        let provider = StorageProvider::from_env();

        // プロバイダーに応じたログ出力
        tracing::info!("Storage provider: {:?}", provider);

        Ok(Self {
            provider,
            endpoint: std::env::var("STORAGE_ENDPOINT").map_err(|_| {
                crate::error::AppError::InternalServerError("STORAGE_ENDPOINT not set".to_string())
            })?,
            bucket: std::env::var("STORAGE_BUCKET").map_err(|_| {
                crate::error::AppError::InternalServerError("STORAGE_BUCKET not set".to_string())
            })?,
            region: std::env::var("STORAGE_REGION").map_err(|_| {
                crate::error::AppError::InternalServerError("STORAGE_REGION not set".to_string())
            })?,
            access_key: std::env::var("STORAGE_ACCESS_KEY").map_err(|_| {
                crate::error::AppError::InternalServerError(
                    "STORAGE_ACCESS_KEY not set".to_string(),
                )
            })?,
            secret_key: std::env::var("STORAGE_SECRET_KEY").map_err(|_| {
                crate::error::AppError::InternalServerError(
                    "STORAGE_SECRET_KEY not set".to_string(),
                )
            })?,
        })
    }
}

/// ストレージサービスのファクトリ関数
pub async fn create_storage_service(config: StorageConfig) -> AppResult<Arc<dyn StorageService>> {
    match config.provider {
        StorageProvider::MinIO | StorageProvider::R2 => {
            // MinIOもR2もS3互換なので同じ実装を使用
            let service = S3StorageService::new(config).await?;
            Ok(Arc::new(service))
        }
    }
}

use chrono::Utc;

/// ファイルサニタイゼーション用のヘルパー関数
pub fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_' || *c == ' ')
        .collect::<String>()
        .trim()
        .to_string()
}
