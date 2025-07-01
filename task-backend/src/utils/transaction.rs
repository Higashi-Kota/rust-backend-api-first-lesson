// task-backend/src/utils/transaction.rs

//! トランザクション管理の統一化
//!
//! 全てのサービス層で一貫したトランザクション管理を提供します。

use crate::error::AppError;
use sea_orm::{DatabaseConnection, DatabaseTransaction, TransactionTrait};
use std::future::Future;
use tracing::{debug, error, info, instrument, warn};

// =============================================================================
// トランザクション管理トレイト
// =============================================================================

/// トランザクション実行を抽象化するトレイト
pub trait TransactionManager {
    /// トランザクション内で操作を実行
    #[allow(clippy::manual_async_fn)]
    fn execute_in_transaction<F, R>(
        &self,
        operation: F,
    ) -> impl std::future::Future<Output = Result<R, AppError>> + Send
    where
        F: for<'c> FnOnce(&'c DatabaseTransaction) -> BoxFuture<'c, Result<R, AppError>>
            + Send
            + 'static,
        R: Send + 'static;
}

// Future型エイリアス（Boxed Future）
type BoxFuture<'a, T> = std::pin::Pin<Box<dyn Future<Output = T> + Send + 'a>>;

// DatabaseConnection への実装
impl TransactionManager for DatabaseConnection {
    #[instrument(skip(self, operation), name = "database_transaction")]
    #[allow(clippy::manual_async_fn)]
    fn execute_in_transaction<F, R>(
        &self,
        operation: F,
    ) -> impl std::future::Future<Output = Result<R, AppError>> + Send
    where
        F: for<'c> FnOnce(&'c DatabaseTransaction) -> BoxFuture<'c, Result<R, AppError>>
            + Send
            + 'static,
        R: Send + 'static,
    {
        async move {
            let transaction_start = std::time::Instant::now();

            debug!("Starting database transaction");

            let txn = self.begin().await.map_err(|e| {
                error!(error = %e, "Failed to begin transaction");
                AppError::InternalServerError("Failed to begin transaction".to_string())
            })?;

            let result = operation(&txn).await;

            match result {
                Ok(value) => {
                    debug!("Transaction operation successful, committing");
                    txn.commit().await.map_err(|e| {
                        error!(error = %e, "Failed to commit transaction");
                        AppError::InternalServerError("Failed to commit transaction".to_string())
                    })?;

                    let duration = transaction_start.elapsed();
                    info!(
                        duration_ms = duration.as_millis(),
                        "Transaction completed successfully"
                    );

                    Ok(value)
                }
                Err(app_error) => {
                    warn!(error = %app_error, "Transaction operation failed, rolling back");

                    if let Err(rollback_error) = txn.rollback().await {
                        error!(
                            original_error = %app_error,
                            rollback_error = %rollback_error,
                            "Failed to rollback transaction"
                        );
                        return Err(AppError::InternalServerError(
                            "Transaction failed and rollback also failed".to_string(),
                        ));
                    }

                    let duration = transaction_start.elapsed();
                    warn!(
                        duration_ms = duration.as_millis(),
                        "Transaction rolled back"
                    );

                    Err(app_error)
                }
            }
        }
    }
}

// =============================================================================
// サービス層トランザクション管理
// =============================================================================

/// サービス層でのトランザクション管理を提供するトレイト
pub trait ServiceTransactionManager {
    /// 複数のリポジトリ操作を単一のトランザクションで実行
    #[allow(clippy::manual_async_fn)]
    fn execute_service_transaction<F, R>(
        &self,
        operation: F,
    ) -> impl std::future::Future<Output = Result<R, AppError>> + Send
    where
        F: for<'c> FnOnce(&'c DatabaseTransaction) -> BoxFuture<'c, Result<R, AppError>>
            + Send
            + 'static,
        R: Send + 'static;
}

// DatabaseConnection への実装
impl ServiceTransactionManager for DatabaseConnection {
    #[allow(clippy::manual_async_fn)]
    fn execute_service_transaction<F, R>(
        &self,
        operation: F,
    ) -> impl std::future::Future<Output = Result<R, AppError>> + Send
    where
        F: for<'c> FnOnce(&'c DatabaseTransaction) -> BoxFuture<'c, Result<R, AppError>>
            + Send
            + 'static,
        R: Send + 'static,
    {
        self.execute_in_transaction(operation)
    }
}

// =============================================================================
// 便利なマクロ
// =============================================================================

/// トランザクション内での操作を簡単に記述するマクロ
#[macro_export]
macro_rules! with_transaction {
    ($db:expr, |$txn:ident| $body:expr) => {{
        use $crate::utils::transaction::TransactionManager;

        $db.execute_in_transaction(move |$txn| Box::pin($body))
            .await
    }};
}

/// サービス層でのトランザクション実行マクロ
#[macro_export]
macro_rules! with_service_transaction {
    ($db:expr, |$txn:ident| $body:expr) => {{
        use $crate::utils::transaction::ServiceTransactionManager;

        $db.execute_service_transaction(move |$txn| Box::pin($body))
            .await
    }};
}

// =============================================================================
// リトライ機能付きトランザクション
// =============================================================================

/// リトライ設定
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 100,
            max_delay_ms: 1000,
        }
    }
}

/// リトライ機能付きトランザクション実行
pub async fn execute_with_retry<F, R>(
    db: &DatabaseConnection,
    operation: F,
    config: RetryConfig,
) -> Result<R, AppError>
where
    F: Clone
        + for<'c> Fn(&'c DatabaseTransaction) -> BoxFuture<'c, Result<R, AppError>>
        + Send
        + 'static,
    R: Send + 'static,
{
    let mut attempt = 1;

    loop {
        let result = db.execute_in_transaction(operation.clone()).await;

        match result {
            Ok(value) => return Ok(value),
            Err(e) => {
                if attempt >= config.max_attempts {
                    error!(
                        attempts = attempt,
                        error = %e,
                        "Transaction failed after all retry attempts"
                    );
                    return Err(e);
                }

                // デッドロックやタイムアウトの場合のみリトライ
                if should_retry(&e) {
                    let delay = calculate_delay(attempt, &config);
                    warn!(
                        attempt = attempt,
                        max_attempts = config.max_attempts,
                        delay_ms = delay,
                        error = %e,
                        "Transaction failed, retrying"
                    );

                    tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                    attempt += 1;
                } else {
                    return Err(e);
                }
            }
        }
    }
}

/// エラーがリトライ可能かどうかを判定
fn should_retry(error: &AppError) -> bool {
    match error {
        AppError::DbErr(db_err) => {
            match db_err {
                sea_orm::DbErr::Conn(_) => true, // 接続エラー
                sea_orm::DbErr::Exec(_) => true, // 実行エラー（デッドロック等）
                _ => false,
            }
        }
        _ => false,
    }
}

/// 指数バックオフでディレイを計算
fn calculate_delay(attempt: u32, config: &RetryConfig) -> u64 {
    let delay = config.base_delay_ms * (2u64.pow(attempt - 1));
    delay.min(config.max_delay_ms)
}

// =============================================================================
// テスト
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.base_delay_ms, 100);
        assert_eq!(config.max_delay_ms, 1000);
    }

    #[test]
    fn test_calculate_delay() {
        let config = RetryConfig::default();

        assert_eq!(calculate_delay(1, &config), 100);
        assert_eq!(calculate_delay(2, &config), 200);
        assert_eq!(calculate_delay(3, &config), 400);
        assert_eq!(calculate_delay(10, &config), 1000); // max_delay_ms で制限
    }

    #[test]
    fn test_should_retry() {
        // リトライ不可能なエラー
        let validation_error = AppError::ValidationError("test".to_string());
        assert!(!should_retry(&validation_error));

        let not_found_error = AppError::NotFound("test".to_string());
        assert!(!should_retry(&not_found_error));
    }
}
