# ストレージプロバイダー設定ガイド

## 概要

Task Backend は複数のS3互換ストレージプロバイダーをサポートしています：

1. **MinIO** - ローカル開発用のS3互換オブジェクトストレージ
2. **Cloudflare R2** - 本番環境用のS3互換オブジェクトストレージ（エグレス料金無料）
3. **Amazon S3** - 将来的なサポート予定

## 設定方法

### 1. MinIO（ローカル開発用）

MinIOは開発環境でS3互換ストレージをローカルで実行するためのソリューションです。

#### MinIOの起動

Docker Composeでの起動：

```yaml
# docker-compose.yml
version: "3.8"
services:
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

  # バケット初期化用
  minio-mc:
    image: minio/mc:latest
    depends_on:
      - minio
    entrypoint: >
      /bin/sh -c "
      mc alias set local http://minio:9000 minioadmin minioadmin;
      mc mb local/task-attachments || true;
      mc anonymous set public local/task-attachments || true;
      exit 0;
      "
    networks:
      - task-network
```

```bash
docker-compose up minio minio-mc -d
```

#### .env設定

```bash
# .env
STORAGE_PROVIDER=minio

# MinIO Configuration
STORAGE_ENDPOINT=http://localhost:9000
STORAGE_BUCKET=task-attachments
STORAGE_REGION=us-east-1
STORAGE_ACCESS_KEY=minioadmin
STORAGE_SECRET_KEY=minioadmin
```

#### MinIO管理コンソール

アクセス: http://localhost:9001
- Username: `minioadmin`
- Password: `minioadmin`

### 2. Cloudflare R2（本番環境用）

Cloudflare R2はS3互換のオブジェクトストレージで、エグレス料金が無料です。

#### R2セットアップ手順

1. **Cloudflareダッシュボードでの設定**

   a. [Cloudflareダッシュボード](https://dash.cloudflare.com/)にログイン
   
   b. 左側メニューから「R2」を選択
   
   c. 「Create bucket」をクリック
   
   d. バケット設定：
      - Bucket name: `task-attachments`（または任意の名前）
      - Location: Automatic（推奨）
      - Default encryption: Enabled（推奨）
   
   e. 「Create bucket」をクリックして作成

2. **APIトークンの作成**

   a. R2ダッシュボードで「Manage R2 API tokens」をクリック
   
   b. 「Create API token」をクリック
   
   c. トークン設定：
      - Token name: `task-backend-production`
      - Permissions: `Object Read & Write`
      - Specify bucket: 作成したバケットを選択
      - TTL: 無期限（または適切な期限）
   
   d. 「Create API Token」をクリック
   
   e. 表示される以下の情報を保存：
      - Token value（Secret Key）
      - Access Key ID
      - Endpoint URL

3. **CORSポリシーの設定**（必要な場合）

   a. バケットの設定ページに移動
   
   b. 「Settings」タブを選択
   
   c. 「CORS Policy」セクションで「Add CORS policy」をクリック
   
   d. 以下のようなポリシーを設定：
   ```json
   [
     {
       "AllowedOrigins": ["https://yourdomain.com"],
       "AllowedMethods": ["GET", "PUT", "POST", "DELETE", "HEAD"],
       "AllowedHeaders": ["*"],
       "ExposeHeaders": ["ETag"],
       "MaxAgeSeconds": 3600
     }
   ]
   ```
   
   **重要**: 
   - `AllowedOrigins`には**フロントエンドのホスティングURL**を指定します
   - バックエンドサーバーはCORSの対象外（サーバー間通信のため）
   - 例：
     - フロントエンド: `https://app.yourdomain.com` → これを指定
     - バックエンド: `https://api.yourdomain.com` → 指定不要
   - 開発環境では `http://localhost:3000` など、フロントエンドの開発サーバーURLを追加

#### .env設定

```bash
# .env
STORAGE_PROVIDER=r2

# Cloudflare R2 Configuration
STORAGE_ENDPOINT=https://YOUR_ACCOUNT_ID.r2.cloudflarestorage.com
STORAGE_BUCKET=task-attachments
STORAGE_REGION=auto
STORAGE_ACCESS_KEY=your-r2-access-key-id
STORAGE_SECRET_KEY=your-r2-secret-access-key
```

## 環境別推奨設定

### 開発環境

```bash
# 開発時はMinIOを使用
STORAGE_PROVIDER=minio
STORAGE_ENDPOINT=http://localhost:9000
STORAGE_BUCKET=task-attachments
STORAGE_REGION=us-east-1
STORAGE_ACCESS_KEY=minioadmin
STORAGE_SECRET_KEY=minioadmin
```

### ステージング環境

```bash
# ステージングでもR2を使用（別バケット）
STORAGE_PROVIDER=r2
STORAGE_ENDPOINT=https://YOUR_ACCOUNT_ID.r2.cloudflarestorage.com
STORAGE_BUCKET=task-attachments-staging
STORAGE_REGION=auto
STORAGE_ACCESS_KEY=your-staging-access-key
STORAGE_SECRET_KEY=your-staging-secret-key
```

### 本番環境

```bash
# 本番ではR2を使用
STORAGE_PROVIDER=r2
STORAGE_ENDPOINT=https://YOUR_ACCOUNT_ID.r2.cloudflarestorage.com
STORAGE_BUCKET=task-attachments-production
STORAGE_REGION=auto
STORAGE_ACCESS_KEY=your-production-access-key
STORAGE_SECRET_KEY=your-production-secret-key
```

## 自動プロバイダー選択

環境変数`APP_ENV`に基づく自動選択：

```bash
APP_ENV=development → MinIO
APP_ENV=staging     → R2
APP_ENV=production  → R2
```

明示的に`STORAGE_PROVIDER`を設定すると、自動選択より優先されます。

## ローカル開発でR2を使用する場合

開発環境でもR2を使用してテストする場合：

```bash
# 開発環境でR2を使用
STORAGE_PROVIDER=r2
STORAGE_ENDPOINT=https://YOUR_ACCOUNT_ID.r2.cloudflarestorage.com
STORAGE_BUCKET=task-attachments-dev
STORAGE_REGION=auto
STORAGE_ACCESS_KEY=your-dev-access-key
STORAGE_SECRET_KEY=your-dev-secret-key
```

## 動作確認

### 1. MinIOでの確認

```bash
# サーバー起動
make dev

# ファイルアップロードテスト
TOKEN=$(curl -s -X POST http://localhost:5000/auth/signin \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123"}' \
  | jq -r '.data.access_token')

TASK_ID=$(curl -s -X GET http://localhost:5000/tasks \
  -H "Authorization: Bearer $TOKEN" \
  | jq -r '.data.tasks[0].id')

# テストファイルアップロード
curl -X POST "http://localhost:5000/tasks/$TASK_ID/attachments" \
  -H "Authorization: Bearer $TOKEN" \
  -F "file=@test-file.pdf"

# MinIO Console でファイル確認
open http://localhost:9001
```

### 2. R2での確認

```bash
# R2設定でサーバー起動
STORAGE_PROVIDER=r2 make run

# 同様のアップロードテスト実行

# Cloudflareダッシュボードでファイル確認
# R2 > Your Bucket > Objects
```

## CORSポリシーについての詳細説明

### なぜフロントエンドのURLを指定するのか

CORSポリシーは、ブラウザのセキュリティ機能であり、**ブラウザで実行されるJavaScript（フロントエンド）**が異なるオリジン（ドメイン）のリソースにアクセスする際に適用されます。

#### 典型的なアーキテクチャ

```
[ユーザーのブラウザ]
    ↓
[フロントエンド] https://app.yourdomain.com
    ↓ (APIリクエスト)
[バックエンド] https://api.yourdomain.com
    ↓ (S3 API)
[R2/MinIO] https://YOUR_ACCOUNT_ID.r2.cloudflarestorage.com
```

#### CORSが必要なケース

1. **直接アップロード**: フロントエンドから直接R2/MinIOにファイルをアップロード
   ```javascript
   // フロントエンドのコード例
   const presignedUrl = await getPresignedUrlFromBackend();
   await fetch(presignedUrl, {
     method: 'PUT',
     body: file
   });
   ```

2. **直接ダウンロード**: フロントエンドから直接R2/MinIOのファイルを取得
   ```javascript
   // 画像プレビューなど
   const imageUrl = await getSignedDownloadUrl();
   imgElement.src = imageUrl; // ブラウザが直接R2にアクセス
   ```

#### CORSが不要なケース

1. **バックエンド経由**: すべてのストレージアクセスをバックエンド経由で行う場合
   ```
   フロントエンド → バックエンド → R2/MinIO
   ```
   この場合、ブラウザはバックエンドとのみ通信するため、ストレージのCORS設定は不要

### 推奨設定例

#### 開発環境
```json
[
  {
    "AllowedOrigins": [
      "http://localhost:3000",
      "http://127.0.0.1:3000"
    ],
    "AllowedMethods": ["GET", "PUT", "POST", "DELETE", "HEAD"],
    "AllowedHeaders": ["*"],
    "ExposeHeaders": ["ETag", "Content-Length"],
    "MaxAgeSeconds": 3600
  }
]
```

#### 本番環境
```json
[
  {
    "AllowedOrigins": [
      "https://app.yourdomain.com",
      "https://www.yourdomain.com"
    ],
    "AllowedMethods": ["GET", "PUT", "POST", "DELETE", "HEAD"],
    "AllowedHeaders": ["*"],
    "ExposeHeaders": ["ETag", "Content-Length"],
    "MaxAgeSeconds": 86400
  }
]
```

### セキュリティ上の注意点

1. **ワイルドカード（*）の使用は避ける**
   ```json
   // ❌ 悪い例
   "AllowedOrigins": ["*"]
   
   // ✅ 良い例
   "AllowedOrigins": ["https://app.yourdomain.com"]
   ```

2. **必要最小限のメソッドのみ許可**
   ```json
   // 読み取り専用の場合
   "AllowedMethods": ["GET", "HEAD"]
   
   // アップロードも必要な場合
   "AllowedMethods": ["GET", "PUT", "HEAD"]
   ```

3. **署名付きURLと組み合わせて使用**
   - 直接アクセスする場合でも、必ず署名付きURLを使用
   - URLには有効期限を設定（推奨: 1時間以内）

## トラブルシューティング

### MinIO関連

```bash
# MinIOコンテナの状態確認
docker ps | grep minio

# MinIOログ確認
docker logs task-minio

# バケットの存在確認
mc alias set local http://localhost:9000 minioadmin minioadmin
mc ls local/
```

### R2関連

1. **接続エラー**:
   - エンドポイントURLが正しいか確認
   - アクセスキーとシークレットキーが正しいか確認
   - APIトークンの権限が適切か確認

2. **バケットアクセスエラー**:
   - バケット名が正しいか確認
   - APIトークンが該当バケットへのアクセス権限を持っているか確認

3. **リージョンエラー**:
   - R2の場合、リージョンは`auto`を使用

### 一般的な問題

1. **ファイルアップロードエラー**:
   ```bash
   # 環境変数の確認
   echo $STORAGE_PROVIDER
   echo $STORAGE_ENDPOINT
   
   # 接続テスト（aws-cli使用）
   aws s3 ls s3://$STORAGE_BUCKET \
     --endpoint-url $STORAGE_ENDPOINT \
     --region $STORAGE_REGION
   ```

2. **署名エラー**:
   - システム時刻が正しいか確認
   - アクセスキー/シークレットキーのスペースや改行を確認

## プロバイダー間の違い

### MinIO
- **利点**: ローカル実行、無料、完全なS3互換性
- **制限**: 開発環境のみ、パフォーマンスは限定的
- **用途**: 開発、テスト

### Cloudflare R2
- **利点**: エグレス料金無料、グローバルCDN、高可用性
- **制限**: 一部のS3 APIは未サポート、リージョンは`auto`固定
- **用途**: 本番環境、大量のダウンロードがある場合

### 互換性に関する注意

R2で未サポートの機能：
- オブジェクトのバージョニング（2024年現在、限定サポート）
- 一部の高度なS3 API（ACL、ライフサイクルポリシーなど）
- リージョン指定（常に`auto`を使用）

## Docker Composeでの完全設定

```yaml
version: "3.8"
services:
  app:
    build: .
    ports:
      - "5000:5000"
    environment:
      - STORAGE_PROVIDER=minio
      - STORAGE_ENDPOINT=http://minio:9000
      - STORAGE_BUCKET=task-attachments
      - STORAGE_REGION=us-east-1
      - STORAGE_ACCESS_KEY=minioadmin
      - STORAGE_SECRET_KEY=minioadmin
    depends_on:
      - postgres
      - minio

  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: taskdb
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: password
    ports:
      - "5432:5432"

  minio:
    image: minio/minio:latest
    ports:
      - "9000:9000"
      - "9001:9001"
    environment:
      MINIO_ROOT_USER: minioadmin
      MINIO_ROOT_PASSWORD: minioadmin
    command: server /data --console-address ":9001"
    volumes:
      - minio_data:/data

volumes:
  minio_data:
```

この設定により、開発環境でMinIOを使ったファイルストレージのテストができます。