// tests/common/mock_storage.rs

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use task_backend::error::AppResult;
use task_backend::features::storage::services::storage::StorageService;

/// テスト用のモックストレージサービス
#[derive(Clone)]
pub struct MockStorageService {
    storage: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl MockStorageService {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl StorageService for MockStorageService {
    async fn store(&self, key: &str, data: &[u8], _content_type: &str) -> AppResult<String> {
        let mut storage = self.storage.lock().unwrap();
        storage.insert(key.to_string(), data.to_vec());
        Ok(key.to_string())
    }

    async fn retrieve(&self, key: &str) -> AppResult<Vec<u8>> {
        let storage = self.storage.lock().unwrap();
        storage
            .get(key)
            .cloned()
            .ok_or_else(|| task_backend::error::AppError::NotFound("File not found".to_string()))
    }

    async fn delete(&self, key: &str) -> AppResult<()> {
        let mut storage = self.storage.lock().unwrap();
        storage.remove(key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> AppResult<bool> {
        let storage = self.storage.lock().unwrap();
        Ok(storage.contains_key(key))
    }

    async fn generate_download_url(&self, key: &str, expires_in_seconds: u64) -> AppResult<String> {
        // モックストレージではダミーの署名付きURLを返す
        Ok(format!(
            "http://mock-storage.local/download/{}?X-Amz-Algorithm=AWS4-HMAC-SHA256&expires={}",
            key, expires_in_seconds
        ))
    }

    async fn generate_upload_url(&self, key: &str, expires_in_seconds: u64) -> AppResult<String> {
        // モックストレージではダミーの署名付きアップロードURLを返す
        Ok(format!(
            "http://mock-storage.local/upload/{}?X-Amz-Algorithm=AWS4-HMAC-SHA256&expires={}",
            key, expires_in_seconds
        ))
    }
}
