use crate::error::AppResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Storage configuration
#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    pub backend: StorageBackend,
    pub upload_path: String,
}

impl StorageConfig {
    pub fn from_env() -> Result<Self, std::env::VarError> {
        // Simple implementation for now
        Ok(Self {
            backend: StorageBackend::Local,
            upload_path: std::env::var("UPLOAD_PATH").unwrap_or_else(|_| "./uploads".to_string()),
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    Local,
    S3,
}

/// Storage service trait
#[async_trait]
pub trait StorageService: Send + Sync {
    /// Store a file and return its storage key
    async fn store(&self, key: &str, data: &[u8], content_type: &str) -> AppResult<String>;

    /// Retrieve a file by its storage key
    async fn retrieve(&self, key: &str) -> AppResult<Vec<u8>>;

    /// Delete a file by its storage key
    async fn delete(&self, key: &str) -> AppResult<()>;

    /// Check if a file exists
    async fn exists(&self, key: &str) -> AppResult<bool>;

    /// Generate a signed URL for direct download
    async fn generate_download_url(&self, key: &str, expires_in_seconds: u64) -> AppResult<String>;

    /// Generate a signed URL for direct upload
    async fn generate_upload_url(&self, key: &str, expires_in_seconds: u64) -> AppResult<String>;
}

/// Local file system storage implementation
pub struct LocalStorageService {
    _upload_path: String,
    files: Arc<RwLock<std::collections::HashMap<String, Vec<u8>>>>,
}

impl LocalStorageService {
    pub fn new(upload_path: String) -> Self {
        Self {
            _upload_path: upload_path,
            files: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }
}

#[async_trait]
impl StorageService for LocalStorageService {
    async fn store(&self, key: &str, data: &[u8], _content_type: &str) -> AppResult<String> {
        let mut files = self.files.write().await;
        files.insert(key.to_string(), data.to_vec());
        Ok(key.to_string())
    }

    async fn retrieve(&self, key: &str) -> AppResult<Vec<u8>> {
        let files = self.files.read().await;
        files
            .get(key)
            .cloned()
            .ok_or_else(|| crate::error::AppError::NotFound("File not found".to_string()))
    }

    async fn delete(&self, key: &str) -> AppResult<()> {
        let mut files = self.files.write().await;
        files.remove(key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> AppResult<bool> {
        let files = self.files.read().await;
        Ok(files.contains_key(key))
    }

    async fn generate_download_url(
        &self,
        key: &str,
        _expires_in_seconds: u64,
    ) -> AppResult<String> {
        // For local storage, just return a path
        Ok(format!("/download/{}", key))
    }

    async fn generate_upload_url(&self, key: &str, _expires_in_seconds: u64) -> AppResult<String> {
        // For local storage, just return a path
        Ok(format!("/upload/{}", key))
    }
}

/// Create storage service based on configuration
pub async fn create_storage_service(config: StorageConfig) -> AppResult<Arc<dyn StorageService>> {
    match config.backend {
        StorageBackend::Local => Ok(Arc::new(LocalStorageService::new(config.upload_path))),
        StorageBackend::S3 => {
            // TODO: Implement S3 storage service
            unimplemented!("S3 storage backend is not yet implemented")
        }
    }
}

/// Sanitize filename to prevent path traversal attacks
pub fn sanitize_filename(filename: &str) -> String {
    // Remove any path separators and other potentially dangerous characters
    filename
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
        .collect::<String>()
        .trim_start_matches('.')
        .to_string()
}
