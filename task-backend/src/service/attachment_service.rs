// task-backend/src/service/attachment_service.rs

use crate::api::dto::attachment_dto::{AttachmentSortBy, SortOrder};
use crate::db::DbPool;
use crate::domain::attachment_share_link_model;
use crate::domain::subscription_tier::SubscriptionTier;
use crate::domain::task_attachment_model::{
    self, get_all_allowed_mime_types, is_allowed_mime_type, MAX_FILE_SIZE_ENTERPRISE,
    MAX_FILE_SIZE_FREE, MAX_FILE_SIZE_PRO,
};
use crate::error::{AppError, AppResult};
use crate::repository::attachment_repository::{AttachmentRepository, CreateAttachmentDto};
use crate::repository::attachment_share_link_repository::{
    AttachmentShareLinkRepository, CreateShareLinkDto,
};
use crate::repository::task_repository::TaskRepository;
use crate::repository::user_repository::UserRepository;
use crate::service::storage_service::{sanitize_filename, StorageService};
use crate::utils::image_optimizer::{is_image_mime_type, optimize_image, ImageOptimizationConfig};
use crate::utils::token::generate_secure_token;
use chrono::{Duration, Utc};
use std::sync::Arc;
use uuid::Uuid;

pub struct AttachmentService {
    attachment_repo: Arc<AttachmentRepository>,
    task_repo: Arc<TaskRepository>,
    user_repo: Arc<UserRepository>,
    share_link_repo: Arc<AttachmentShareLinkRepository>,
    storage: Arc<dyn StorageService>,
}

impl AttachmentService {
    pub fn new(db_pool: DbPool, storage: Arc<dyn StorageService>) -> Self {
        Self {
            attachment_repo: Arc::new(AttachmentRepository::new(db_pool.clone())),
            task_repo: Arc::new(TaskRepository::new(db_pool.clone())),
            user_repo: Arc::new(UserRepository::new(db_pool.clone())),
            share_link_repo: Arc::new(AttachmentShareLinkRepository::new(db_pool)),
            storage,
        }
    }

    /// ファイルをアップロード
    pub async fn upload_file(
        &self,
        task_id: Uuid,
        user_id: Uuid,
        file_name: String,
        file_data: Vec<u8>,
        mime_type: String,
    ) -> AppResult<task_attachment_model::Model> {
        // MIMEタイプが許可されているかチェック
        if !is_allowed_mime_type(&mime_type) {
            let allowed_types = get_all_allowed_mime_types();
            return Err(AppError::BadRequest(format!(
                "File type '{}' is not allowed. Supported types: {}",
                mime_type,
                allowed_types.join(", ")
            )));
        }

        // タスクの存在と権限チェック
        self.check_task_access(task_id, user_id).await?;

        // ファイルサイズチェック
        let file_size = file_data.len() as i64;
        self.validate_file_size(user_id, file_size).await?;

        // ファイル名のサニタイゼーション
        let sanitized_filename = sanitize_filename(&file_name);
        if sanitized_filename.is_empty() {
            return Err(AppError::BadRequest("Invalid file name".to_string()));
        }

        // ストレージ使用量チェック
        self.check_storage_quota(user_id, file_size).await?;

        // 画像の場合は最適化を実行
        let (final_data, final_mime_type, final_file_name, original_storage_key) =
            if is_image_mime_type(&mime_type) {
                // ユーザーのサブスクリプションに基づいて設定を決定
                let optimization_config = self.get_optimization_config_for_user(user_id).await?;

                match optimize_image(
                    &file_data,
                    &mime_type,
                    &sanitized_filename,
                    &optimization_config,
                ) {
                    Ok(result) => {
                        tracing::info!(
                            "Image optimized: {} -> {} bytes ({}% reduction)",
                            result.original_size,
                            result.optimized_size,
                            result.compression_ratio
                        );

                        // 元のファイルも保持する場合
                        let original_key = if optimization_config.keep_original {
                            let key =
                                self.storage
                                    .upload(file_data, &mime_type)
                                    .await
                                    .map_err(|e| {
                                        AppError::InternalServerError(format!(
                                            "Failed to upload original file: {}",
                                            e
                                        ))
                                    })?;
                            Some(key)
                        } else {
                            None
                        };

                        (
                            result.data,
                            result.mime_type,
                            result.file_name,
                            original_key,
                        )
                    }
                    Err(e) => {
                        // 最適化に失敗した場合は元のファイルをそのまま使用
                        tracing::warn!("Image optimization failed: {}, using original", e);
                        (
                            file_data,
                            mime_type.clone(),
                            sanitized_filename.clone(),
                            None,
                        )
                    }
                }
            } else {
                (
                    file_data,
                    mime_type.clone(),
                    sanitized_filename.clone(),
                    None,
                )
            };

        // 最適化後のファイルサイズを再計算
        let final_file_size = final_data.len() as i64;

        // ストレージにアップロード
        let storage_key = self
            .storage
            .upload(final_data, &final_mime_type)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to upload file: {}", e)))?;

        // メタデータをデータベースに保存
        let attachment = self
            .attachment_repo
            .create(CreateAttachmentDto {
                task_id,
                uploaded_by: user_id,
                file_name: final_file_name,
                file_size: final_file_size,
                mime_type: final_mime_type,
                storage_key: storage_key.clone(),
            })
            .await
            .map_err(|e| {
                // エラーが発生した場合、アップロードしたファイルを削除（ベストエフォート）
                let storage = self.storage.clone();
                let key = storage_key.clone();
                let original_key = original_storage_key.clone();
                tokio::spawn(async move {
                    let _ = storage.delete(&key).await;
                    if let Some(orig_key) = original_key {
                        let _ = storage.delete(&orig_key).await;
                    }
                });
                AppError::DbErr(e)
            })?;

        Ok(attachment)
    }

    /// 添付ファイルをダウンロード
    pub async fn download_attachment(
        &self,
        attachment_id: Uuid,
        user_id: Uuid,
    ) -> AppResult<(Vec<u8>, String, String)> {
        // アタッチメント情報を取得
        let attachment = self
            .attachment_repo
            .find_by_id(attachment_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Attachment not found".to_string()))?;

        // アクセス権限チェック
        self.check_task_access(attachment.task_id, user_id).await?;

        // ファイルの存在確認
        let exists = self
            .storage
            .exists(&attachment.storage_key)
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Failed to check file existence: {}", e))
            })?;

        if !exists {
            return Err(AppError::NotFound("File not found in storage".to_string()));
        }

        // ストレージからダウンロード
        let file_data = self
            .storage
            .download(&attachment.storage_key)
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Failed to download file: {}", e))
            })?;

        Ok((file_data, attachment.file_name, attachment.mime_type))
    }

    /// 添付ファイルを削除
    pub async fn delete_attachment(&self, attachment_id: Uuid, user_id: Uuid) -> AppResult<()> {
        // アタッチメント情報を取得
        let attachment = self
            .attachment_repo
            .find_by_id(attachment_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Attachment not found".to_string()))?;

        // 削除権限チェック（アップロード者またはタスク所有者）
        self.check_delete_permission(user_id, &attachment).await?;

        // ストレージから削除
        self.storage
            .delete(&attachment.storage_key)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to delete file: {}", e)))?;

        // データベースから削除
        self.attachment_repo.delete(attachment_id).await?;

        Ok(())
    }

    /// タスクの添付ファイル一覧を取得（ページング付き）
    pub async fn list_task_attachments_paginated(
        &self,
        task_id: Uuid,
        user_id: Uuid,
        page: u64,
        per_page: u64,
        sort_by: Option<AttachmentSortBy>,
        sort_order: Option<SortOrder>,
    ) -> AppResult<(Vec<task_attachment_model::Model>, u64)> {
        // アクセス権限チェック
        self.check_task_access(task_id, user_id).await?;

        // ページングのバリデーション
        if page == 0 {
            return Err(AppError::BadRequest(
                "Page must be greater than 0".to_string(),
            ));
        }
        if per_page > 100 {
            return Err(AppError::BadRequest(
                "Per page must not exceed 100".to_string(),
            ));
        }

        // 添付ファイル一覧を取得
        let (attachments, total) = self
            .attachment_repo
            .find_by_task_id_paginated(task_id, page, per_page, sort_by, sort_order)
            .await?;

        Ok((attachments, total))
    }

    /// ファイルサイズのバリデーション
    async fn validate_file_size(&self, user_id: Uuid, file_size: i64) -> AppResult<()> {
        let max_size = self.get_max_file_size_for_user(user_id).await?;

        if file_size > max_size {
            return Err(AppError::BadRequest(format!(
                "File size exceeds the limit of {} MB",
                max_size / (1024 * 1024)
            )));
        }

        Ok(())
    }

    /// ユーザーの最大ファイルサイズを取得
    async fn get_max_file_size_for_user(&self, user_id: Uuid) -> AppResult<i64> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        let tier = SubscriptionTier::from_str(&user.subscription_tier).ok_or_else(|| {
            AppError::InternalServerError("Invalid subscription tier".to_string())
        })?;

        let max_size = match tier {
            SubscriptionTier::Free => MAX_FILE_SIZE_FREE,
            SubscriptionTier::Pro => MAX_FILE_SIZE_PRO,
            SubscriptionTier::Enterprise => MAX_FILE_SIZE_ENTERPRISE,
        };

        Ok(max_size)
    }

    /// ストレージ使用量のチェック
    async fn check_storage_quota(&self, user_id: Uuid, file_size: i64) -> AppResult<()> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        let tier = SubscriptionTier::from_str(&user.subscription_tier).ok_or_else(|| {
            AppError::InternalServerError("Invalid subscription tier".to_string())
        })?;

        // Enterpriseプランは無制限
        if tier == SubscriptionTier::Enterprise {
            return Ok(());
        }

        let current_usage = self
            .attachment_repo
            .calculate_user_storage_usage(user_id)
            .await?;

        let max_storage = match tier {
            SubscriptionTier::Free => 100 * 1024 * 1024, // 100MB
            SubscriptionTier::Pro => 10 * 1024 * 1024 * 1024, // 10GB
            SubscriptionTier::Enterprise => i64::MAX,    // 無制限
        };

        if current_usage + file_size > max_storage {
            return Err(AppError::BadRequest(format!(
                "Storage quota exceeded. Current usage: {} MB, Limit: {} MB",
                current_usage / (1024 * 1024),
                max_storage / (1024 * 1024)
            )));
        }

        Ok(())
    }

    /// タスクへのアクセス権限チェック
    async fn check_task_access(&self, task_id: Uuid, user_id: Uuid) -> AppResult<()> {
        let task = self
            .task_repo
            .find_by_id(task_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

        // タスクのユーザーIDと一致するかチェック
        if let Some(task_user_id) = task.user_id {
            if task_user_id != user_id {
                return Err(AppError::Forbidden(
                    "You don't have permission to access this task".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// 削除権限チェック
    async fn check_delete_permission(
        &self,
        user_id: Uuid,
        attachment: &task_attachment_model::Model,
    ) -> AppResult<()> {
        // アップロード者の場合は削除可能
        if attachment.uploaded_by == user_id {
            return Ok(());
        }

        // タスクの所有者の場合も削除可能
        let task = self
            .task_repo
            .find_by_id(attachment.task_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

        if let Some(task_user_id) = task.user_id {
            if task_user_id == user_id {
                return Ok(());
            }
        }

        Err(AppError::Forbidden(
            "You don't have permission to delete this attachment".to_string(),
        ))
    }

    /// 添付ファイルの署名付きダウンロードURLを生成
    pub async fn generate_download_url(
        &self,
        attachment_id: Uuid,
        user_id: Uuid,
        expires_in_seconds: u64,
    ) -> AppResult<String> {
        // アタッチメント情報を取得
        let attachment = self
            .attachment_repo
            .find_by_id(attachment_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Attachment not found".to_string()))?;

        // アクセス権限チェック
        self.check_task_access(attachment.task_id, user_id).await?;

        // 有効期限の検証（最小1分、最大24時間）
        let expires_in = expires_in_seconds.clamp(60, 86400);

        // 署名付きURLを生成
        let url = self
            .storage
            .generate_download_url(&attachment.storage_key, expires_in)
            .await?;

        Ok(url)
    }

    /// 外部共有リンクを作成
    pub async fn create_share_link(
        &self,
        attachment_id: Uuid,
        user_id: Uuid,
        description: Option<String>,
        expires_in_hours: u32,
        max_access_count: Option<i32>,
    ) -> AppResult<attachment_share_link_model::Model> {
        // アタッチメント情報を取得
        let attachment = self
            .attachment_repo
            .find_by_id(attachment_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Attachment not found".to_string()))?;

        // アクセス権限チェック
        self.check_task_access(attachment.task_id, user_id).await?;

        // 有効期限を計算（最小1時間、最大30日）
        let expires_in_hours = expires_in_hours.clamp(1, 24 * 30);
        let expires_at = Utc::now() + Duration::hours(expires_in_hours as i64);

        // 共有トークンを生成
        let share_token = generate_secure_token(32);

        // 共有リンクを作成
        let share_link = self
            .share_link_repo
            .create(CreateShareLinkDto {
                id: Uuid::new_v4(),
                attachment_id,
                created_by: user_id,
                share_token,
                description,
                expires_at,
                max_access_count,
            })
            .await?;

        Ok(share_link)
    }

    /// 共有リンクでファイルをダウンロード（認証不要）
    pub async fn download_via_share_link(
        &self,
        share_token: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> AppResult<(Vec<u8>, String, String)> {
        // 共有リンクを取得
        let share_link = self
            .share_link_repo
            .find_by_token(share_token)
            .await?
            .ok_or_else(|| AppError::NotFound("Invalid share link".to_string()))?;

        // 有効性チェック
        if share_link.is_revoked {
            return Err(AppError::Forbidden(
                "Share link has been revoked".to_string(),
            ));
        }

        if share_link.expires_at < Utc::now() {
            return Err(AppError::Forbidden("Share link has expired".to_string()));
        }

        if let Some(max_count) = share_link.max_access_count {
            if share_link.current_access_count >= max_count {
                return Err(AppError::Forbidden(
                    "Share link has reached maximum access count".to_string(),
                ));
            }
        }

        // アタッチメント情報を取得
        let attachment = self
            .attachment_repo
            .find_by_id(share_link.attachment_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Attachment not found".to_string()))?;

        // ファイルをダウンロード
        let file_data = self
            .storage
            .download(&attachment.storage_key)
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Failed to download file: {}", e))
            })?;

        // アクセスを記録（エラーがあってもダウンロードは続行）
        let _ = self
            .share_link_repo
            .record_access(share_link.id, ip_address, user_agent)
            .await;

        Ok((file_data, attachment.file_name, attachment.mime_type))
    }

    /// 共有リンクの一覧を取得
    pub async fn list_share_links(
        &self,
        attachment_id: Uuid,
        user_id: Uuid,
    ) -> AppResult<Vec<attachment_share_link_model::Model>> {
        // アタッチメント情報を取得
        let attachment = self
            .attachment_repo
            .find_by_id(attachment_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Attachment not found".to_string()))?;

        // アクセス権限チェック
        self.check_task_access(attachment.task_id, user_id).await?;

        // 共有リンク一覧を取得
        self.share_link_repo
            .find_by_attachment_id(attachment_id)
            .await
    }

    /// 共有リンクを無効化
    pub async fn revoke_share_link(&self, share_link_id: Uuid, user_id: Uuid) -> AppResult<()> {
        // 共有リンクを取得
        let share_link = self
            .share_link_repo
            .find_by_id(share_link_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Share link not found".to_string()))?;

        // アタッチメント情報を取得
        let attachment = self
            .attachment_repo
            .find_by_id(share_link.attachment_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Attachment not found".to_string()))?;

        // アクセス権限チェック（作成者またはタスク所有者）
        if share_link.created_by != user_id {
            self.check_task_access(attachment.task_id, user_id).await?;
        }

        // 共有リンクを無効化
        self.share_link_repo.revoke(share_link_id).await
    }

    /// ユーザーのサブスクリプションに基づいて画像最適化設定を取得
    async fn get_optimization_config_for_user(
        &self,
        user_id: Uuid,
    ) -> AppResult<ImageOptimizationConfig> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        let tier = SubscriptionTier::from_str(&user.subscription_tier).ok_or_else(|| {
            AppError::InternalServerError("Invalid subscription tier".to_string())
        })?;

        let config = match tier {
            SubscriptionTier::Free => ImageOptimizationConfig {
                enable_webp_conversion: true,
                max_width: 1280,
                max_height: 1280,
                webp_quality: 75.0,
                jpeg_quality: 75,
                keep_original: false,
            },
            SubscriptionTier::Pro => ImageOptimizationConfig {
                enable_webp_conversion: true,
                max_width: 2048,
                max_height: 2048,
                webp_quality: 85.0,
                jpeg_quality: 85,
                keep_original: false,
            },
            SubscriptionTier::Enterprise => ImageOptimizationConfig {
                enable_webp_conversion: true,
                max_width: 4096,
                max_height: 4096,
                webp_quality: 90.0,
                jpeg_quality: 90,
                keep_original: true,
            },
        };

        Ok(config)
    }

    /// 直接アップロード用の署名付きURLを生成
    pub async fn generate_upload_url(
        &self,
        key: &str,
        expires_in_seconds: u64,
    ) -> AppResult<String> {
        self.storage
            .generate_upload_url(key, expires_in_seconds)
            .await
    }
}
