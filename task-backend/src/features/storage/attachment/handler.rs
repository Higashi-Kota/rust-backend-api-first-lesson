// task-backend/src/features/storage/attachment/handler.rs

use crate::api::AppState;
use crate::error::{AppError, AppResult};
use crate::features::auth::middleware::AuthenticatedUser;
use crate::shared::types::common::ApiResponse;
use crate::shared::types::pagination::PaginatedResponse;
use axum::body::Body;
use axum::extract::DefaultBodyLimit;
use axum::{
    extract::{Json, Multipart, Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Router,
};
use chrono::{Duration, Utc};
use tracing::info;
use uuid::Uuid;

use super::dto::{
    AttachmentDto, AttachmentFilterDto, AttachmentUploadResponse, CreateShareLinkRequest,
    CreateShareLinkResponse, GenerateDownloadUrlRequest, GenerateDownloadUrlResponse,
    GenerateUploadUrlRequest, GenerateUploadUrlResponse, ShareLinkDto, ShareLinkListResponse,
};

/// ファイルアップロードハンドラー
pub async fn upload_attachment_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path(task_id): Path<Uuid>,
    mut multipart: Multipart,
) -> AppResult<impl IntoResponse> {
    let attachment_service = &app_state.attachment_service;
    info!(
        user_id = %user.user_id(),
        task_id = %task_id,
        "Starting file upload"
    );

    // multipartデータを処理
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to read multipart data: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" {
            // ファイル名を取得
            let file_name = field
                .file_name()
                .ok_or_else(|| AppError::BadRequest("File name is required".to_string()))?
                .to_string();

            // Content-Typeを取得（なければmimeライブラリで推測）
            let content_type = field
                .content_type()
                .map(|ct| ct.to_string())
                .or_else(|| {
                    // ファイル拡張子からMIMEタイプを推測
                    file_name
                        .split('.')
                        .next_back()
                        .and_then(mime_guess::from_ext)
                        .map(|mime| mime.to_string())
                })
                .unwrap_or_else(|| "application/octet-stream".to_string());

            // ファイルデータを読み込む
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(format!("Failed to read file data: {}", e)))?;

            // AttachmentServiceを使用してアップロード
            let attachment = attachment_service
                .upload_file(
                    task_id,
                    user.user_id(),
                    file_name,
                    data.to_vec(),
                    content_type,
                )
                .await?;

            let response = AttachmentUploadResponse {
                attachment: attachment.into(),
                message: "File uploaded successfully".to_string(),
            };

            return Ok((
                StatusCode::CREATED,
                Json(ApiResponse::success("File uploaded successfully", response)),
            ));
        }
    }

    Err(AppError::BadRequest("No file provided".to_string()))
}

/// タスクの添付ファイル一覧取得ハンドラー
pub async fn list_attachments_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path(task_id): Path<Uuid>,
    Query(filter): Query<AttachmentFilterDto>,
) -> AppResult<impl IntoResponse> {
    info!(
        user_id = %user.user_id(),
        task_id = %task_id,
        "Listing attachments for task"
    );

    let attachment_service = &app_state.attachment_service;

    let page = filter.page.unwrap_or(1);
    let per_page = filter.per_page.unwrap_or(20).min(100); // 最大100件まで

    // ページング付きで取得
    let (attachments, total) = attachment_service
        .list_task_attachments_paginated(
            task_id,
            user.user_id(),
            page,
            per_page,
            filter.sort_by,
            filter.sort_order,
        )
        .await?;

    let attachments_dto: Vec<AttachmentDto> = attachments.into_iter().map(Into::into).collect();

    let response =
        PaginatedResponse::new(attachments_dto, page as i32, per_page as i32, total as i64);

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(
            "Attachments retrieved successfully",
            response,
        )),
    ))
}

/// 添付ファイルダウンロードハンドラー
pub async fn download_attachment_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path(attachment_id): Path<Uuid>,
) -> AppResult<Response> {
    let attachment_service = &app_state.attachment_service;

    info!(
        user_id = %user.user_id(),
        attachment_id = %attachment_id,
        "Downloading attachment"
    );

    // AttachmentServiceを使用してダウンロード
    let (file_data, file_name, mime_type) = attachment_service
        .download_attachment(attachment_id, user.user_id())
        .await?;

    // ヘッダーを設定
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, mime_type.parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", file_name)
            .parse()
            .unwrap(),
    );

    // レスポンスを構築
    Ok((StatusCode::OK, headers, Body::from(file_data)).into_response())
}

/// 添付ファイル削除ハンドラー
pub async fn delete_attachment_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path(attachment_id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let attachment_service = &app_state.attachment_service;

    info!(
        user_id = %user.user_id(),
        attachment_id = %attachment_id,
        "Deleting attachment"
    );

    // AttachmentServiceを使用して削除
    attachment_service
        .delete_attachment(attachment_id, user.user_id())
        .await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("Attachment deleted successfully", ())),
    ))
}

/// 署名付きダウンロードURL生成ハンドラー
pub async fn generate_download_url_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path(attachment_id): Path<Uuid>,
    Query(request): Query<GenerateDownloadUrlRequest>,
) -> AppResult<impl IntoResponse> {
    let attachment_service = &app_state.attachment_service;

    info!(
        user_id = %user.user_id(),
        attachment_id = %attachment_id,
        "Generating download URL"
    );

    // リクエストから有効期限を取得（デフォルト1時間）
    let expires_in_seconds = request.expires_in_seconds.unwrap_or(3600);

    // 署名付きURLを生成
    let download_url = attachment_service
        .generate_download_url(attachment_id, user.user_id(), expires_in_seconds)
        .await?;

    // レスポンスを構築
    let response = GenerateDownloadUrlResponse {
        download_url,
        expires_in_seconds,
        expires_at: Utc::now() + Duration::seconds(expires_in_seconds as i64),
    };

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(
            "Download URL generated successfully",
            response,
        )),
    ))
}

/// 共有リンクを作成
pub async fn create_share_link_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path(attachment_id): Path<Uuid>,
    Json(request): Json<CreateShareLinkRequest>,
) -> AppResult<impl IntoResponse> {
    let attachment_service = &app_state.attachment_service;

    info!(
        user_id = %user.user_id(),
        attachment_id = %attachment_id,
        "Creating share link"
    );

    // デフォルト値を使用
    let expires_in_hours = request.expires_in_hours.unwrap_or(24);

    // 共有リンクを作成
    let share_link = attachment_service
        .create_share_link(
            attachment_id,
            user.user_id(),
            request.description,
            expires_in_hours,
            request.max_access_count,
        )
        .await?;

    // ベースURLを構築
    let base_url = format!("http://{}", app_state.server_addr);
    let share_link_dto = ShareLinkDto::from_model(share_link, &base_url);

    let response = CreateShareLinkResponse {
        share_link: share_link_dto,
        message: "Share link created successfully".to_string(),
    };

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            "Share link created successfully",
            response,
        )),
    ))
}

/// 共有リンク一覧を取得
pub async fn list_share_links_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path(attachment_id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let attachment_service = &app_state.attachment_service;

    info!(
        user_id = %user.user_id(),
        attachment_id = %attachment_id,
        "Listing share links"
    );

    let share_links = attachment_service
        .list_share_links(attachment_id, user.user_id())
        .await?;

    let base_url = format!("http://{}", app_state.server_addr);
    let share_link_dtos: Vec<ShareLinkDto> = share_links
        .into_iter()
        .map(|link| ShareLinkDto::from_model(link, &base_url))
        .collect();

    let response = ShareLinkListResponse {
        total: share_link_dtos.len(),
        share_links: share_link_dtos,
    };

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(
            "Share links retrieved successfully",
            response,
        )),
    ))
}

/// 共有リンクを無効化
pub async fn revoke_share_link_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path(share_link_id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let attachment_service = &app_state.attachment_service;

    info!(
        user_id = %user.user_id(),
        share_link_id = %share_link_id,
        "Revoking share link"
    );

    attachment_service
        .revoke_share_link(share_link_id, user.user_id())
        .await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("Share link revoked successfully", ())),
    ))
}

/// 共有リンクでファイルをダウンロード（認証不要）
pub async fn download_via_share_link_handler(
    State(app_state): State<AppState>,
    Path(share_token): Path<String>,
    headers: HeaderMap,
) -> AppResult<Response> {
    let attachment_service = &app_state.attachment_service;

    // IPアドレスとUser-Agentを取得
    let ip_address = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .or_else(|| headers.get("x-real-ip").and_then(|v| v.to_str().ok()))
        .map(|s| s.to_string());

    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    info!(
        share_token = %share_token,
        ip_address = ?ip_address,
        "Downloading via share link"
    );

    // 共有リンクでダウンロード
    let (file_data, file_name, mime_type) = attachment_service
        .download_via_share_link(&share_token, ip_address, user_agent)
        .await?;

    // ヘッダーを設定
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, mime_type.parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", file_name)
            .parse()
            .unwrap(),
    );

    // レスポンスを構築
    Ok((StatusCode::OK, headers, Body::from(file_data)).into_response())
}

/// 添付ファイル関連のルーティング設定
pub fn attachment_routes() -> Router<AppState> {
    Router::new()
        // タスクに添付ファイルをアップロード
        .route(
            "/tasks/{task_id}/attachments",
            post(upload_attachment_handler)
                // 最大500MBまでのファイルを許可（Enterprise向け。実際の制限はサービス層で行う）
                .layer(DefaultBodyLimit::max(500 * 1024 * 1024)),
        )
        // タスクの添付ファイル一覧を取得
        .route(
            "/tasks/{task_id}/attachments",
            get(list_attachments_handler),
        )
        // 添付ファイルをダウンロード
        .route(
            "/attachments/{attachment_id}",
            get(download_attachment_handler),
        )
        // 添付ファイルを削除
        .route(
            "/attachments/{attachment_id}",
            delete(delete_attachment_handler),
        )
        // 署名付きダウンロードURLを生成
        .route(
            "/attachments/{attachment_id}/download-url",
            get(generate_download_url_handler),
        )
        // 共有リンクを作成
        .route(
            "/attachments/{attachment_id}/share-links",
            post(create_share_link_handler),
        )
        // 共有リンク一覧を取得
        .route(
            "/attachments/{attachment_id}/share-links",
            get(list_share_links_handler),
        )
        // 共有リンクを無効化
        .route(
            "/share-links/{share_link_id}",
            delete(revoke_share_link_handler),
        )
        // 共有リンクでダウンロード（認証不要）
        .route("/share/{share_token}", get(download_via_share_link_handler))
        // 直接アップロード用の署名付きURL生成
        .route(
            "/tasks/{task_id}/attachments/upload-url",
            post(generate_upload_url_handler),
        )
}

/// 直接アップロード用の署名付きURL生成ハンドラー
pub async fn generate_upload_url_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path(task_id): Path<Uuid>,
    Json(request): Json<GenerateUploadUrlRequest>,
) -> AppResult<Json<ApiResponse<GenerateUploadUrlResponse>>> {
    info!(
        user_id = %user.user_id(),
        task_id = %task_id,
        file_name = %request.file_name,
        "Generating upload URL"
    );

    // ユーザーがタスクへのアクセス権限を持っているか確認
    let task_service = &app_state.task_service;
    let _task = task_service
        .get_task_for_user(user.user_id(), task_id)
        .await?;

    // キーを生成（タスクID/ユーザーID/タイムスタンプ/ファイル名）
    let timestamp = Utc::now().timestamp();
    let upload_key = format!(
        "tasks/{}/attachments/{}/{}/{}",
        task_id,
        user.user_id(),
        timestamp,
        request.file_name
    );

    // 有効期限を検証
    let expires_in_seconds = request.expires_in_seconds.unwrap_or(3600).clamp(60, 3600); // 最小60秒、最大1時間

    // アップロードURLを生成
    let upload_url = app_state
        .attachment_service
        .generate_upload_url(&upload_key, expires_in_seconds)
        .await?;

    let expires_at = Utc::now() + Duration::seconds(expires_in_seconds as i64);

    Ok(Json(ApiResponse::success(
        "Upload URL generated successfully",
        GenerateUploadUrlResponse {
            upload_url,
            upload_key,
            expires_at,
        },
    )))
}

// mime_guessクレートの代替実装（既存のmimeクレートを使用）
mod mime_guess {
    pub fn from_ext(ext: &str) -> Option<mime::Mime> {
        match ext.to_lowercase().as_str() {
            // 画像ファイル
            "jpg" | "jpeg" => Some(mime::IMAGE_JPEG),
            "png" => Some(mime::IMAGE_PNG),
            "gif" => Some(mime::IMAGE_GIF),
            "webp" => Some("image/webp".parse().ok()?),
            // ドキュメントファイル
            "pdf" => Some("application/pdf".parse().ok()?),
            "doc" => Some("application/msword".parse().ok()?),
            "docx" => Some(
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                    .parse()
                    .ok()?,
            ),
            "xls" => Some("application/vnd.ms-excel".parse().ok()?),
            "xlsx" => Some(
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                    .parse()
                    .ok()?,
            ),
            "csv" => Some("text/csv".parse().ok()?),
            _ => None,
        }
    }
}
