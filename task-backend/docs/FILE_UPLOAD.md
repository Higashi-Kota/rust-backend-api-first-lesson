# タスクアタッチメント機能実装ガイド

このドキュメントでは、タスクに複数のファイルをアップロードできるアタッチメント機能の実装について説明します。

## 📋 目次

1. [概要](#概要)
2. [システム設計](#システム設計)
3. [実装済み機能](#実装済み機能)
4. [ローカル開発環境セットアップ](#ローカル開発環境セットアップ)
5. [動作確認手順](#動作確認手順)
6. [セキュリティ考慮事項](#セキュリティ考慮事項)
7. [テスト方法](#テスト方法)
8. [トラブルシューティング](#トラブルシューティング)
9. [今後の拡張予定](#今後の拡張予定)

## 概要

### 機能要件

- タスクに複数のファイルを添付可能
- ファイルのアップロード、ダウンロード、削除機能
- サブスクリプションプランに応じたファイルサイズ制限とストレージクォータ
- セキュアなファイルアクセス制御
- 将来的な拡張: 署名付きURL、自動圧縮によるストレージ最適化

### 技術スタック

- **ストレージ**: 
  - 開発環境: MinIO（S3互換）
  - 本番環境: Cloudflare R2
- **ファイルアップロード**: Axumのmultipart機能
- **データベース**: PostgreSQL（メタデータ保存）
- **AWSクライアント**: aws-sdk-s3（MinIO/R2両対応）

## システム設計

### 1. データベース設計

#### 新規テーブル: `task_attachments`

```sql
CREATE TABLE task_attachments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    uploaded_by UUID NOT NULL REFERENCES users(id),
    file_name VARCHAR(255) NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    storage_key VARCHAR(500) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- インデックス
CREATE INDEX idx_task_attachments_task_id ON task_attachments(task_id);
CREATE INDEX idx_task_attachments_uploaded_by ON task_attachments(uploaded_by);
```

### 2. API設計

#### エンドポイント一覧

| メソッド | パス | 説明 | 認証 |
|---------|------|------|------|
| POST | `/tasks/{task_id}/attachments` | ファイルアップロード | 必要 |
| GET | `/tasks/{task_id}/attachments` | アタッチメント一覧取得 | 必要 |
| GET | `/attachments/{attachment_id}` | ファイルダウンロード（サーバー経由） | 必要 |
| DELETE | `/attachments/{attachment_id}` | ファイル削除 | 必要 |
| GET | `/attachments/{attachment_id}/download-url` | 署名付きダウンロードURL生成 | 必要 |
| POST | `/attachments/{attachment_id}/share-links` | 外部共有リンク作成 | 必要 |
| GET | `/attachments/{attachment_id}/share-links` | 共有リンク一覧取得 | 必要 |
| DELETE | `/share-links/{share_link_id}` | 共有リンク無効化 | 必要 |
| GET | `/share/{share_token}` | 共有リンクでダウンロード | **不要** |

**注意**: パスパラメータは `{param}` 形式を使用（Axum 0.8の仕様）

#### リクエスト/レスポンス例

**ファイルアップロード**
```bash
POST /tasks/{task_id}/attachments
Content-Type: multipart/form-data

file: (binary)
```

**レスポンス**
```json
{
  "success": true,
  "message": "File uploaded successfully",
  "data": {
    "attachment": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "task_id": "660e8400-e29b-41d4-a716-446655440000",
      "file_name": "設計書.pdf",
      "file_size": 1048576,
      "mime_type": "application/pdf",
      "uploaded_by": "770e8400-e29b-41d4-a716-446655440000",
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z"
    },
    "message": "File uploaded successfully"
  }
}
```

**署名付きURL生成**
```bash
GET /attachments/{attachment_id}/download-url?expires_in_seconds=3600
Authorization: Bearer <token>
```

**レスポンス**
```json
{
  "success": true,
  "message": "Download URL generated successfully",
  "data": {
    "download_url": "https://storage.example.com/task-attachments/attachments/550e8400-e29b-41d4-a716-446655440000?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=...",
    "expires_in_seconds": 3600,
    "expires_at": "2024-01-01T01:00:00Z"
  }
}
```

**外部共有リンク作成**
```bash
POST /attachments/{attachment_id}/share-links
Authorization: Bearer <token>
Content-Type: application/json

{
  "description": "Client presentation materials",
  "expires_in_hours": 72,
  "max_access_count": 10
}
```

**レスポンス**
```json
{
  "success": true,
  "message": "Share link created successfully",
  "data": {
    "share_link": {
      "id": "770e8400-e29b-41d4-a716-446655440000",
      "attachment_id": "550e8400-e29b-41d4-a716-446655440000",
      "description": "Client presentation materials",
      "share_token": "AbCdEfGhIjKlMnOpQrStUvWxYz123456",
      "share_url": "http://localhost:3000/share/AbCdEfGhIjKlMnOpQrStUvWxYz123456",
      "expires_at": "2024-01-04T00:00:00Z",
      "max_access_count": 10,
      "current_access_count": 0,
      "is_revoked": false,
      "created_at": "2024-01-01T00:00:00Z"
    },
    "message": "Share link created successfully"
  }
}
```

**共有リンクでダウンロード（認証不要）**
```bash
GET /share/AbCdEfGhIjKlMnOpQrStUvWxYz123456
# 認証ヘッダー不要
```

### 3. ファイルサイズ制限

| プラン | 最大ファイルサイズ | 最大ストレージ容量 |
|--------|-------------------|-------------------|
| Free | 5MB | 100MB |
| Pro | 50MB | 10GB |
| Enterprise | 500MB | 無制限 |

### 4. サポートファイル形式

#### 現在サポートされている形式:
- **画像**: JPEG, PNG, GIF, WebP
- **ドキュメント**: PDF, Word (.doc, .docx), Excel (.xls, .xlsx), CSV

## 実装済み機能

### 完了済みの実装

1. ✅ データベーススキーマ（task_attachmentsテーブル）
2. ✅ ドメインモデル（TaskAttachment）
3. ✅ ストレージサービス（S3StorageService）
4. ✅ ストレージプロバイダー切り替え機能（MinIO/R2）
5. ✅ アタッチメントサービス（AttachmentService）
6. ✅ APIハンドラー（upload, list, download, delete, generate-url）
7. ✅ ルーティング設定
8. ✅ 統合テスト（17件のテストケース）
9. ✅ ファイルタイプ拡張（画像 + PDF/Word/Excel/CSV）
10. ✅ サブスクリプション階層別ファイルサイズ制限
11. ✅ ストレージクォータ管理
12. ✅ 詳細なエラーハンドリング
13. ✅ 署名付きダウンロードURL生成（Phase 2）
14. ✅ 外部共有リンク機能（Phase 2）
15. ✅ 共有リンクアクセスログ（Phase 2）

### アーキテクチャの特徴

- **サービス層分離**: ストレージサービスとアタッチメントサービスを分離
- **リポジトリパターン**: データアクセスをAttachmentRepositoryで抽象化
- **エラーハンドリング**: AppErrorによる統一的なエラー処理

## ローカル開発環境セットアップ

### Step 1: MinIOのセットアップ

#### Docker Composeを使用する場合（推奨）

`docker-compose.yml`に以下を追加済み:

```yaml
  minio:
    image: minio/minio:latest
    container_name: task-minio
    ports:
      - "9000:9000"     # API endpoint
      - "9001:9001"     # Console endpoint
    environment:
      MINIO_ROOT_USER: minioadmin
      MINIO_ROOT_PASSWORD: minioadmin
    command: server /data --console-address ":9001"
    volumes:
      - minio_data:/data
    networks:
      - task-network

volumes:
  minio_data:
```

起動方法:
```bash
# MinIOを含む全サービスを起動
docker-compose up -d

# MinIOのみ起動
docker-compose up -d minio
```

#### Dockerコマンドで個別に起動する場合

```bash
# MinIOコンテナを起動
docker run -d \
  -p 9000:9000 \
  -p 9001:9001 \
  --name task-minio \
  -e MINIO_ROOT_USER=minioadmin \
  -e MINIO_ROOT_PASSWORD=minioadmin \
  minio/minio server /data --console-address ":9001"
```

### Step 2: MinIOクライアント（mc）のインストール

```bash
# Linuxの場合
wget https://dl.min.io/client/mc/release/linux-amd64/mc
chmod +x mc
sudo mv mc /usr/local/bin/

# macOSの場合
brew install minio/stable/mc

# Windowsの場合はMinIOのサイトからダウンロード
```

### Step 3: バケットの作成

```bash
# MinIOのエイリアスを設定
mc alias set local http://localhost:9000 minioadmin minioadmin

# バケットを作成
mc mb local/task-attachments

# バケットの一覧を確認
mc ls local/
```

### Step 4: 環境変数の設定

`.env`ファイルに以下を追加:

```bash
# Storage Configuration (MinIO for development)
STORAGE_ENDPOINT=http://localhost:9000
STORAGE_BUCKET=task-attachments
STORAGE_REGION=us-east-1
STORAGE_ACCESS_KEY=minioadmin
STORAGE_SECRET_KEY=minioadmin
```

**重要**: これらの環境変数は必須です。設定されていない場合、アプリケーションの起動に失敗します。

## 動作確認手順

### 1. アプリケーションの起動

```bash
# データベースマイグレーションを実行
make db-migrate

# アプリケーションを起動
make dev
```

正常起動時:
```
📦 Initializing storage service...
✅ Storage service initialized successfully
```

環境変数が不足している場合、アプリケーションは起動せずエラーメッセージが表示されます。

### 2. MinIO管理画面での確認

1. ブラウザで http://localhost:9001 にアクセス
2. ログイン情報:
   - Username: `minioadmin`
   - Password: `minioadmin`
3. `task-attachments`バケットが作成されていることを確認

### 3. APIエンドポイントのテスト

#### 完全なフロー: アカウント作成 → サインイン → タスク作成 → ファイルアップロード

```bash
# Step 1: アカウント作成
curl -X POST http://localhost:3000/auth/signup \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "username": "testuser",
    "password": "SecurePass123!"
  }'

# Step 2: サインイン（トークン取得）
TOKEN=$(curl -s -X POST http://localhost:3000/auth/signin \
  -H "Content-Type: application/json" \
  -d '{
    "identifier": "test@example.com",
    "password": "SecurePass123!"
  }' | jq -r '.tokens.access_token')

# Step 3: タスク作成
TASK_RESPONSE=$(curl -s -X POST http://localhost:3000/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "テストタスク",
    "description": "ファイルアップロードテスト用",
    "status": "todo"
  }')

# タスクIDを取得
TASK_ID=$(echo $TASK_RESPONSE | jq -r '.id')

# Step 4: テスト用画像ファイルを作成
echo -e "\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x02\x00\x00\x00\x90wS\xde" > test-image.png

# Step 5: ファイルアップロード
curl -X POST "http://localhost:3000/tasks/$TASK_ID/attachments" \
  -H "Authorization: Bearer $TOKEN" \
  -F "file=@test-image.png"
```

成功時のレスポンス例:
```json
{
  "success": true,
  "message": "File uploaded successfully",
  "data": {
    "attachment": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "task_id": "660e8400-e29b-41d4-a716-446655440000",
      "file_name": "test-image.png",
      "file_size": 1048576,
      "mime_type": "image/png",
      "uploaded_by": "770e8400-e29b-41d4-a716-446655440000",
      "created_at": "2025-01-04T00:00:00Z",
      "updated_at": "2025-01-04T00:00:00Z"
    },
    "message": "File uploaded successfully"
  }
}
```

#### アタッチメント一覧取得

```bash
curl -X GET "http://localhost:3000/tasks/$TASK_ID/attachments?page=1&per_page=20" \
  -H "Authorization: Bearer $TOKEN"
```

レスポンス例:
```json
{
  "success": true,
  "data": {
    "attachments": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "task_id": "660e8400-e29b-41d4-a716-446655440000",
        "uploaded_by": "770e8400-e29b-41d4-a716-446655440000",
        "file_name": "test-image.png",
        "file_size": 1048576,
        "mime_type": "image/png",
        "created_at": "2025-01-04T00:00:00Z",
        "updated_at": "2025-01-04T00:00:00Z"
      }
    ],
    "total": 1
  },
  "message": null
}
```

#### ファイルダウンロード（サーバー経由）

```bash
# アタッチメントIDを取得
ATTACHMENT_ID=$(curl -s -X GET "http://localhost:3000/tasks/$TASK_ID/attachments" \
  -H "Authorization: Bearer $TOKEN" \
  | jq -r '.data.attachments[0].id')

# ファイルをダウンロード
curl -X GET "http://localhost:3000/attachments/$ATTACHMENT_ID" \
  -H "Authorization: Bearer $TOKEN" \
  -o downloaded-file.png
```

#### 署名付きURLでダウンロード（推奨）

```bash
# 署名付きURLを取得
DOWNLOAD_URL=$(curl -s -X GET "http://localhost:3000/attachments/$ATTACHMENT_ID/download-url?expires_in_seconds=3600" \
  -H "Authorization: Bearer $TOKEN" \
  | jq -r '.data.download_url')

# 署名付きURLで直接ダウンロード（認証不要）
curl -o downloaded-file-direct.png "$DOWNLOAD_URL"
```

レスポンス例:
```json
{
  "success": true,
  "message": "Download URL generated successfully",
  "data": {
    "download_url": "http://localhost:9000/task-attachments/attachments/550e8400-e29b-41d4-a716-446655440000?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=...",
    "expires_in_seconds": 3600,
    "expires_at": "2025-01-04T01:00:00Z"
  }
}
```

#### ファイル削除

```bash
curl -X DELETE "http://localhost:3000/attachments/$ATTACHMENT_ID" \
  -H "Authorization: Bearer $TOKEN"
```

レスポンス例:
```json
{
  "success": true,
  "message": "Attachment deleted successfully",
  "data": null
}
```

#### 共有リンクの作成と使用

```bash
# Step 1: 新しいファイルをアップロード（共有用）
UPLOAD_RESPONSE=$(curl -s -X POST "http://localhost:3000/tasks/$TASK_ID/attachments" \
  -H "Authorization: Bearer $TOKEN" \
  -F "file=@test-image.png")

ATTACHMENT_ID=$(echo $UPLOAD_RESPONSE | jq -r '.data.attachment.id')

# Step 2: 共有リンクを作成（72時間有効、最大10回アクセス可能）
SHARE_RESPONSE=$(curl -s -X POST "http://localhost:3000/attachments/$ATTACHMENT_ID/share-links" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Client presentation",
    "expires_in_hours": 72,
    "max_access_count": 10
  }')

echo "Share Response: $SHARE_RESPONSE"

# 共有URLを取得
SHARE_URL=$(echo $SHARE_RESPONSE | jq -r '.data.share_link.share_url')
echo "Share URL: $SHARE_URL"

# Step 3: 共有リンクでダウンロード（認証不要）
curl -o shared-file.png "$SHARE_URL"

# Step 4: 共有リンク一覧を確認
curl -X GET "http://localhost:3000/attachments/$ATTACHMENT_ID/share-links" \
  -H "Authorization: Bearer $TOKEN"

# Step 5: 共有リンクを無効化
SHARE_LINK_ID=$(echo $SHARE_RESPONSE | jq -r '.data.share_link.id')
curl -X DELETE "http://localhost:3000/share-links/$SHARE_LINK_ID" \
  -H "Authorization: Bearer $TOKEN"
```

共有リンク作成のレスポンス例:
```json
{
  "success": true,
  "message": "Share link created successfully",
  "data": {
    "share_link": {
      "id": "770e8400-e29b-41d4-a716-446655440000",
      "attachment_id": "550e8400-e29b-41d4-a716-446655440000",
      "description": "Client presentation",
      "share_token": "AbCdEfGhIjKlMnOpQrStUvWxYz123456",
      "share_url": "http://localhost:3000/share/AbCdEfGhIjKlMnOpQrStUvWxYz123456",
      "expires_at": "2025-01-07T00:00:00Z",
      "max_access_count": 10,
      "current_access_count": 0,
      "is_revoked": false,
      "created_at": "2025-01-04T00:00:00Z"
    },
    "message": "Share link created successfully"
  }
}
```

### 4. MinIOでファイルの確認

```bash
# アップロードされたファイルの一覧
mc ls local/task-attachments/attachments/

# 特定のファイルの詳細
mc stat local/task-attachments/attachments/<UUID>
```

## セキュリティ考慮事項

### 1. 現在実装済みのセキュリティ機能

- **認証必須**: 全エンドポイントでJWT認証が必要
- **ファイルサイズ制限**: サブスクリプション階層別（Free: 5MB, Pro: 50MB, Enterprise: 500MB）
- **MIME型検出**: ファイル拡張子からMIME型を推測
- **ファイルタイプ制限**: 画像（JPEG, PNG, GIF, WebP）とドキュメント（PDF, Word, Excel, CSV）のみ許可
- **ユニークストレージキー**: UUIDv4でファイル名の衝突を防止
- **アクセス制御**: タスクへのアクセス権限を持つユーザーのみ操作可能
- **ストレージクォータ管理**: サブスクリプション階層別（Free: 100MB, Pro: 10GB, Enterprise: 無制限）
- **署名付きURL**: 期限付きの直接ダウンロードURL（サーバー負荷軽減）

### 2. 今後実装予定のセキュリティ機能

#### アップロード時の検証強化（Phase 3）

- ファイルヘッダーによる実際のファイル形式検証（マジックナンバーチェック）
- 悪意のあるスクリプトやマクロの検出

## テスト方法

### 1. 統合テストの実行

```bash
# 全テストを実行
make ci-check-fast

# アタッチメント関連のテストのみ実行
cargo test attachment --features test
```

### 2. 実装済みのテストケース

#### 基本機能テスト（17件）
- ✅ 画像アップロード成功（PNG）
- ✅ PDFアップロード成功
- ✅ CSVアップロード成功
- ✅ Wordファイル（.docx）アップロード成功
- ✅ Excelファイル（.xlsx）アップロード成功
- ✅ サポートされていないファイルタイプ拒否（.exe）
- ✅ ファイルサイズ超過エラー（サブスクリプション階層別）
- ✅ ファイルなしエラー
- ✅ 一覧取得（ページング対応）
- ✅ ダウンロード成功
- ✅ 削除成功
- ✅ 権限なしエラー
- ✅ 存在しないファイルエラー
- ✅ 署名付きURL生成成功
- ✅ 署名付きURLデフォルト有効期限
- ✅ 署名付きURL - 存在しないファイル
- ✅ 署名付きURL - 権限なしエラー

#### 共有リンクテスト（10件）
- ✅ 共有リンク作成成功
- ✅ 共有リンクのデフォルト値
- ✅ 共有リンクでのダウンロード成功
- ✅ 共有リンク一覧取得
- ✅ 共有リンク無効化
- ✅ アクセス回数制限動作
- ✅ 無効な共有トークンエラー
- ✅ 他ユーザーの添付ファイルへの共有リンク作成拒否
- ✅ 他ユーザーの共有リンク無効化拒否
- ✅ アクセスログ記録

### 3. テスト用のサンプルファイル作成

```bash
# テスト用の画像ファイルを作成
convert -size 100x100 xc:blue test-image.png

# または、ImageMagickがない場合は小さなファイルを用意
echo "test content" > test-file.txt
```

## トラブルシューティング

### よくある問題と解決方法

#### 1. アプリケーション起動エラー「Failed to load storage configuration」

**原因**: ストレージ環境変数が設定されていない

**解決方法**:
```bash
# .envファイルを確認
cat .env | grep STORAGE_

# 環境変数を設定してから再起動
export STORAGE_ENDPOINT=http://localhost:9000
export STORAGE_BUCKET=task-attachments
export STORAGE_REGION=us-east-1
export STORAGE_ACCESS_KEY=minioadmin
export STORAGE_SECRET_KEY=minioadmin
make dev
```

#### 2. MinIOへの接続エラー

**症状**: `Connection refused`エラー

**解決方法**:
```bash
# MinIOをDockerで起動
docker run -d \
  -p 9000:9000 \
  -p 9001:9001 \
  --name minio \
  -e MINIO_ROOT_USER=minioadmin \
  -e MINIO_ROOT_PASSWORD=minioadmin \
  minio/minio server /data --console-address ":9001"

# バケットを作成
mc alias set local http://localhost:9000 minioadmin minioadmin
mc mb local/task-attachments
```

#### 3. ファイルアップロードが413エラー

**症状**: `Payload Too Large`エラー

**解決方法**:
現在の実装では、attachment_routes()で100MBまでの制限が設定済み:
```rust
post(upload_attachment_handler)
    .layer(DefaultBodyLimit::max(100 * 1024 * 1024))
```

#### 4. CORS エラー

**症状**: ブラウザからのアップロードが失敗

**解決方法**:
現在の実装では、main.rsでCORS設定済み。必要に応じて調整:
```rust
// middleware/auth.rsのcors_layer()関数を確認
```


### デバッグTips

1. **ログレベルを上げる**
   ```bash
   RUST_LOG=debug cargo run
   ```

2. **MinIO管理画面で確認**
   - http://localhost:9001 にアクセス
   - ユーザー名: minioadmin
   - パスワード: minioadmin

3. **データベースの確認**
   ```sql
   SELECT * FROM task_attachments WHERE task_id = 'YOUR_TASK_ID';
   ```

## 今後の拡張予定

### Phase 1（完了）✅

1. **ファイルタイプ拡張** ✅
   - PDF、Word、Excel、CSVサポート済み
   - MIMEタイプのホワイトリスト実装済み

2. **ストレージクォータ管理** ✅
   - ユーザー単位のストレージ使用量追跡済み
   - サブスクリプションに基づく容量制限実装済み

3. **詳細なエラーハンドリング** ✅
   - ファイル形式エラーの具体的なメッセージ実装済み
   - クォータ超過時の詳細情報表示済み

### Phase 2（完了）✅

1. **署名付きURL機能** ✅
   - **一時的なダウンロードURL生成** ✅
     - S3/R2の署名付きURL（Presigned URL）機能を利用 ✅
     - 有効期限設定（デフォルト1時間、最大24時間）✅
     - 直接ストレージアクセスによるサーバー負荷軽減 ✅
     - ダウンロード帯域幅の節約 ✅
   
2. **外部共有リンク機能** ✅
   - **認証不要の期限付き共有URL生成** ✅
     - 独自トークンベースの共有システム ✅
     - 有効期限設定（最小1時間、最大30日）✅
     - `/share/{token}` エンドポイントで公開アクセス可能 ✅
   - **アクセス回数制限オプション** ✅
     - 任意の最大アクセス回数を設定可能 ✅
     - アクセスカウントの自動追跡 ✅
   - **共有リンクの無効化機能** ✅
     - 作成者またはタスク所有者が無効化可能 ✅
     - 無効化後は即座にアクセス不可 ✅
   - **アクセスログ記録** ✅
     - IPアドレスとUser-Agentを記録 ✅
     - アクセス時刻の記録 ✅

### Phase 3（完了）✅

1. **ストレージ最適化** ✅
   - **自動圧縮機能** ✅
     - 画像ファイル: WebP変換による容量削減（最大70%削減）✅
       - JPEG/PNG/GIF → WebP自動変換 ✅
       - サブスクリプション別の品質設定 ✅
         - Free: 75%品質、最大1280x1280px
         - Pro: 85%品質、最大2048x2048px
         - Enterprise: 90%品質、最大4096x4096px、元ファイル保持
     - 圧縮レベルの設定（品質優先/サイズ優先）✅
     - 元ファイル保持オプション（Enterpriseプランのみ）✅
   
   - **アップロード時の最適化** ✅
     - 画像のリサイズオプション（最大解像度設定）✅
       - アスペクト比を維持した自動リサイズ ✅
     - EXIFなどのメタデータ自動除去 ✅
       - Free/Proプランではメタデータを削除
       - Enterpriseプランではメタデータを保持

### 実装を見送る機能

以下の機能は複雑性とコストパフォーマンスを考慮し、実装を見送ります：

- **サムネイル生成**: クライアント側での実装が効率的
- **バージョン管理**: S3/R2のネイティブバージョニング機能で代替可能
- **全文検索**: ファイル名検索で実用上十分、必要に応じて別システムとして実装
- **ウイルススキャン**: セキュリティ専用の別システムとして実装する方が柔軟性が高い

## 参考情報

### SDK互換性

- **MinIO**: aws-sdk-s3で完全対応、`force_path_style(true)`が必要
- **Cloudflare R2**: リージョンを`auto`に設定、バッチ削除は700件まで
- **共通**: S3 APIの主要機能をサポート、IAM APIは非対応

### 関連ドキュメント

- [MinIO公式ドキュメント](https://min.io/docs/)
- [Cloudflare R2ドキュメント](https://developers.cloudflare.com/r2/)
- [AWS SDK for Rust](https://aws.amazon.com/sdk-for-rust/)

このドキュメントは実装の進捗に応じて随時更新されます。