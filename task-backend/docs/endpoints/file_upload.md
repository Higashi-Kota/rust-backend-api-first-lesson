# ファイルアップロード管理エンドポイント

タスクへのファイル添付、ダウンロード、共有リンク作成などのファイル管理機能を提供するAPIエンドポイント群です。動的パーミッションシステムにより、ユーザーのサブスクリプション階層に応じて異なる制限やストレージ容量を提供します。

## ファイルアップロード・管理

### 1. ファイルアップロード (POST /tasks/{task_id}/attachments)

タスクにファイルを添付します。

**リクエスト例:**
```bash
curl -X POST http://localhost:5000/tasks/550e8400-e29b-41d4-a716-446655440001/attachments \
  -H "Authorization: Bearer <access_token>" \
  -F "file=@/path/to/document.pdf"
```

**成功レスポンス例 (201 Created):**
```json
{
  "status": "success",
  "message": "File uploaded successfully",
  "data": {
    "attachment": {
      "id": "660e8400-e29b-41d4-a716-446655440002",
      "task_id": "550e8400-e29b-41d4-a716-446655440001",
      "uploaded_by": "550e8400-e29b-41d4-a716-446655440000",
      "file_name": "document.pdf",
      "file_size": 2048576,
      "mime_type": "application/pdf",
      "created_at": "2025-06-12T10:00:00Z",
      "updated_at": "2025-06-12T10:00:00Z"
    },
    "message": "File uploaded successfully"
  }
}
```

**サブスクリプション別制限:**
- **Free**: 最大10MB/ファイル、総容量100MB
- **Pro**: 最大50MB/ファイル、総容量10GB
- **Enterprise**: 最大500MB/ファイル、総容量無制限

**サポートされるファイル形式:**
- 画像: JPEG, PNG, GIF, WebP
- ドキュメント: PDF, DOC, DOCX, XLS, XLSX, CSV
- その他はサービス設定に依存

### 2. 添付ファイル一覧取得 (GET /tasks/{task_id}/attachments)

指定したタスクの添付ファイル一覧を取得します。

**リクエスト例:**
```bash
# 基本的な一覧取得
curl -X GET http://localhost:5000/tasks/550e8400-e29b-41d4-a716-446655440001/attachments \
  -H "Authorization: Bearer <access_token>"

# ページネーション付き
curl -X GET "http://localhost:5000/tasks/550e8400-e29b-41d4-a716-446655440001/attachments?page=1&per_page=20" \
  -H "Authorization: Bearer <access_token>"

# ソート指定
curl -X GET "http://localhost:5000/tasks/550e8400-e29b-41d4-a716-446655440001/attachments?sort_by=file_size&sort_order=desc" \
  -H "Authorization: Bearer <access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "status": "success",
  "message": "Attachments retrieved successfully",
  "data": {
    "items": [
      {
        "id": "660e8400-e29b-41d4-a716-446655440002",
        "task_id": "550e8400-e29b-41d4-a716-446655440001",
        "uploaded_by": "550e8400-e29b-41d4-a716-446655440000",
        "file_name": "document.pdf",
        "file_size": 2048576,
        "mime_type": "application/pdf",
        "created_at": "2025-06-12T10:00:00Z",
        "updated_at": "2025-06-12T10:00:00Z"
      }
    ],
    "pagination": {
      "current_page": 1,
      "page_size": 20,
      "total_items": 5,
      "total_pages": 1,
      "has_next_page": false,
      "has_previous_page": false
    }
  }
}
```

**クエリパラメータ:**
- `page`: ページ番号（デフォルト: 1）
- `per_page`: 1ページあたりの件数（デフォルト: 20、最大: 100）
- `sort_by`: ソートフィールド（`created_at`, `file_name`, `file_size`）
- `sort_order`: ソート順（`asc`, `desc`）

### 3. ファイルダウンロード (GET /attachments/{attachment_id})

添付ファイルをダウンロードします。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/attachments/660e8400-e29b-41d4-a716-446655440002 \
  -H "Authorization: Bearer <access_token>" \
  -o downloaded_file.pdf
```

**成功レスポンス:** 
- ステータスコード: 200 OK
- Content-Type: ファイルのMIMEタイプ
- Content-Disposition: attachment; filename="document.pdf"
- ボディ: ファイルのバイナリデータ

### 4. ファイル削除 (DELETE /attachments/{attachment_id})

添付ファイルを削除します。

**リクエスト例:**
```bash
curl -X DELETE http://localhost:5000/attachments/660e8400-e29b-41d4-a716-446655440002 \
  -H "Authorization: Bearer <access_token>"
```

**成功レスポンス例 (200 OK):**
```json
{
  "status": "success",
  "message": "Attachment deleted successfully",
  "data": null
}
```

## 署名付きダウンロードURL

### 5. 署名付きダウンロードURL生成 (GET /attachments/{attachment_id}/download-url)

一定期間有効な署名付きダウンロードURLを生成します。外部システムとの連携やブラウザでの直接ダウンロードに使用できます。

**リクエスト例:**
```bash
# デフォルト有効期限（1時間）
curl -X GET http://localhost:5000/attachments/660e8400-e29b-41d4-a716-446655440002/download-url \
  -H "Authorization: Bearer <access_token>"

# カスタム有効期限（24時間）
curl -X GET "http://localhost:5000/attachments/660e8400-e29b-41d4-a716-446655440002/download-url?expires_in_seconds=86400" \
  -H "Authorization: Bearer <access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "status": "success",
  "message": "Download URL generated successfully",
  "data": {
    "download_url": "http://localhost:5000/storage/download/660e8400-e29b-41d4-a716-446655440002?signature=abcdef123456&expires=1735920000",
    "expires_in_seconds": 3600,
    "expires_at": "2025-06-12T11:00:00Z"
  }
}
```

**クエリパラメータ:**
- `expires_in_seconds`: URL有効期限（秒）。最小60秒、最大86400秒（24時間）

## 外部共有機能

### 6. 共有リンク作成 (POST /attachments/{attachment_id}/share-links)

ファイルを外部に共有するためのリンクを作成します。

**リクエスト例:**
```bash
curl -X POST http://localhost:5000/attachments/660e8400-e29b-41d4-a716-446655440002/share-links \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "description": "クライアント様への資料共有",
    "expires_in_hours": 48,
    "max_access_count": 10
  }'
```

**レスポンス例 (201 Created):**
```json
{
  "status": "success",
  "message": "Share link created successfully",
  "data": {
    "share_link": {
      "id": "770e8400-e29b-41d4-a716-446655440003",
      "attachment_id": "660e8400-e29b-41d4-a716-446655440002",
      "created_by": "550e8400-e29b-41d4-a716-446655440000",
      "share_token": "shr_1234567890abcdef",
      "description": "クライアント様への資料共有",
      "expires_at": "2025-06-14T10:00:00Z",
      "max_access_count": 10,
      "current_access_count": 0,
      "is_revoked": false,
      "created_at": "2025-06-12T10:00:00Z",
      "updated_at": "2025-06-12T10:00:00Z",
      "share_url": "http://localhost:5000/share/shr_1234567890abcdef"
    },
    "message": "Share link created successfully"
  }
}
```

**リクエストボディ:**
- `description`: 共有リンクの説明（任意）
- `expires_in_hours`: 有効期限（時間）。最小1時間、最大720時間（30日）
- `max_access_count`: 最大アクセス回数（任意）

### 7. 共有リンク一覧取得 (GET /attachments/{attachment_id}/share-links)

ファイルの共有リンク一覧を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/attachments/660e8400-e29b-41d4-a716-446655440002/share-links \
  -H "Authorization: Bearer <access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "status": "success",
  "message": "Share links retrieved successfully",
  "data": {
    "share_links": [
      {
        "id": "770e8400-e29b-41d4-a716-446655440003",
        "attachment_id": "660e8400-e29b-41d4-a716-446655440002",
        "created_by": "550e8400-e29b-41d4-a716-446655440000",
        "share_token": "shr_1234567890abcdef",
        "description": "クライアント様への資料共有",
        "expires_at": "2025-06-14T10:00:00Z",
        "max_access_count": 10,
        "current_access_count": 3,
        "is_revoked": false,
        "created_at": "2025-06-12T10:00:00Z",
        "updated_at": "2025-06-12T10:00:00Z",
        "share_url": "http://localhost:5000/share/shr_1234567890abcdef"
      }
    ],
    "total": 1
  }
}
```

### 8. 共有リンク無効化 (DELETE /share-links/{share_link_id})

共有リンクを無効化します。

**リクエスト例:**
```bash
curl -X DELETE http://localhost:5000/share-links/770e8400-e29b-41d4-a716-446655440003 \
  -H "Authorization: Bearer <access_token>"
```

**成功レスポンス例 (200 OK):**
```json
{
  "status": "success",
  "message": "Share link revoked successfully",
  "data": null
}
```

### 9. 共有リンクでダウンロード (GET /share/{share_token})

共有リンクを使用してファイルをダウンロードします。**認証不要**のパブリックエンドポイントです。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/share/shr_1234567890abcdef \
  -o shared_file.pdf
```

**成功レスポンス:**
- ステータスコード: 200 OK
- Content-Type: ファイルのMIMEタイプ
- Content-Disposition: attachment; filename="document.pdf"
- ボディ: ファイルのバイナリデータ

**アクセスログ:**
共有リンクへのアクセスは自動的に記録されます（IPアドレス、User-Agent、アクセス日時）。

## 使用例

### 完整なファイル共有ワークフロー

```bash
# アクセストークンを取得済みと仮定
ACCESS_TOKEN="your_access_token_here"
TASK_ID="550e8400-e29b-41d4-a716-446655440001"

# 1. ファイルをアップロード
UPLOAD_RESPONSE=$(curl -s -X POST http://localhost:5000/tasks/$TASK_ID/attachments \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -F "file=@./important_document.pdf")

ATTACHMENT_ID=$(echo $UPLOAD_RESPONSE | jq -r '.data.attachment.id')

# 2. アップロードされたファイル一覧を確認
curl -s -X GET http://localhost:5000/tasks/$TASK_ID/attachments \
  -H "Authorization: Bearer $ACCESS_TOKEN" | jq

# 3. 共有リンクを作成（48時間有効、最大10回アクセス可能）
SHARE_RESPONSE=$(curl -s -X POST http://localhost:5000/attachments/$ATTACHMENT_ID/share-links \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "description": "取引先への資料共有",
    "expires_in_hours": 48,
    "max_access_count": 10
  }')

SHARE_URL=$(echo $SHARE_RESPONSE | jq -r '.data.share_link.share_url')
echo "共有URL: $SHARE_URL"

# 4. 共有URLで直接ダウンロード（認証不要）
curl -X GET $SHARE_URL -o downloaded_via_share.pdf

# 5. 共有リンクの使用状況を確認
curl -s -X GET http://localhost:5000/attachments/$ATTACHMENT_ID/share-links \
  -H "Authorization: Bearer $ACCESS_TOKEN" | jq

# 6. 必要に応じて共有リンクを無効化
SHARE_LINK_ID=$(echo $SHARE_RESPONSE | jq -r '.data.share_link.id')
curl -X DELETE http://localhost:5000/share-links/$SHARE_LINK_ID \
  -H "Authorization: Bearer $ACCESS_TOKEN"
```

## エラーレスポンス例

### ファイルタイプエラー (400 Bad Request)
```json
{
  "error": "File type 'application/x-executable' is not allowed. Supported types: image/jpeg, image/png, image/gif, image/webp, application/pdf, application/msword, application/vnd.openxmlformats-officedocument.wordprocessingml.document, application/vnd.ms-excel, application/vnd.openxmlformats-officedocument.spreadsheetml.sheet, text/csv",
  "error_type": "bad_request"
}
```

### ファイルサイズ超過エラー (400 Bad Request)
```json
{
  "error": "File size exceeds the maximum allowed size of 10 MB for Free tier users",
  "error_type": "bad_request"
}
```

### ストレージ容量超過エラー (400 Bad Request)
```json
{
  "error": "Storage quota exceeded. Current usage: 95 MB, Limit: 100 MB",
  "error_type": "bad_request"
}
```

### 共有リンク期限切れエラー (404 Not Found)
```json
{
  "error": "Share link has expired",
  "error_type": "not_found"
}
```

### 共有リンクアクセス上限エラー (404 Not Found)
```json
{
  "error": "Share link has reached maximum access count",
  "error_type": "not_found"
}
```

## セキュリティと制限事項

### ファイルアップロード制限
- **ファイルサイズ制限**: サブスクリプションレベルに依存
- **ストレージ容量制限**: サブスクリプションレベルに依存
- **許可されるファイル形式**: システム設定に依存
- **同時アップロード数**: リクエストあたり1ファイル

### 画像最適化
画像ファイルは自動的に最適化されます：
- **Free**: 基本的な圧縮のみ
- **Pro**: 高品質圧縮、メタデータ保持
- **Enterprise**: カスタマイズ可能な圧縮設定、元画像も保持

### 共有リンクのセキュリティ
- 推測困難なトークン生成
- 有効期限の自動チェック
- アクセス回数の制限
- アクセスログの記録
- 無効化機能

### レート制限
- ファイルアップロード: 10回/分
- 共有リンク作成: 20回/時間
- ダウンロード: 100回/分