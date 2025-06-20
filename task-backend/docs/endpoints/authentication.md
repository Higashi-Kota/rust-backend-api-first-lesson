# 認証（Authentication）エンドポイント

認証・認可関連のAPIエンドポイント群です。JWT認証を使用したユーザー管理とセッション管理を提供します。

## 認証不要エンドポイント

### 1. ユーザー登録 (POST /auth/signup)

新規ユーザーを登録します。

**リクエスト例:**
```bash
curl -X POST http://localhost:3000/auth/signup \
  -H "Content-Type: application/json" \
  -d '{
    "username": "newuser",
    "email": "newuser@example.com",
    "password": "SecurePass123!"
  }'
```

### 2. ユーザーログイン (POST /auth/signin)

ユーザーをログインしてアクセストークンを取得します。

**リクエスト例:**
```bash
curl -X POST http://localhost:3000/auth/signin \
  -H "Content-Type: application/json" \
  -d '{
    "identifier": "user@example.com",
    "password": "SecurePass123!"
  }'
```

### 3. トークンリフレッシュ (POST /auth/refresh)

リフレッシュトークンを使用してアクセストークンを更新します。

**リクエスト例:**
```bash
curl -X POST http://localhost:3000/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "your_refresh_token_here"
  }'
```

### 4. パスワードリセット要求 (POST /auth/forgot-password)

パスワードリセット用のメールを送信します。

**リクエスト例:**
```bash
curl -X POST http://localhost:3000/auth/forgot-password \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com"
  }'
```

### 5. パスワードリセット実行 (POST /auth/reset-password)

パスワードリセットトークンを使用してパスワードを変更します。

**リクエスト例:**
```bash
curl -X POST http://localhost:3000/auth/reset-password \
  -H "Content-Type: application/json" \
  -d '{
    "token": "reset_token_from_email",
    "new_password": "NewSecurePass456!"
  }'
```

### 6. 認証ステータス確認 (GET /auth/status)

認証システムの状態を確認します。

**リクエスト例:**
```bash
curl -X GET http://localhost:3000/auth/status
```

## 認証必要エンドポイント

以下のエンドポイントには有効なJWTアクセストークンが必要です。

### 7. 現在のユーザー情報取得 (GET /auth/me)

認証済みユーザーの詳細情報を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:3000/auth/me \
  -H "Authorization: Bearer <access_token>"
```

### 8. パスワード変更 (PUT /auth/change-password)

現在のユーザーのパスワードを変更します。

**リクエスト例:**
```bash
curl -X PUT http://localhost:3000/auth/change-password \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "current_password": "CurrentPass123!",
    "new_password": "NewSecurePass456!",
    "new_password_confirmation": "NewSecurePass456!"
  }'
```

### 9. ログアウト (POST /auth/signout)

現在のセッションを終了します。

**リクエスト例:**
```bash
curl -X POST http://localhost:3000/auth/signout \
  -H "Authorization: Bearer <access_token>"
```

### 10. 全デバイスからログアウト (POST /auth/signout-all)

ユーザーのすべてのアクティブセッションを終了します。

**リクエスト例:**
```bash
curl -X POST http://localhost:3000/auth/signout-all \
  -H "Authorization: Bearer <access_token>"
```

### 11. アカウント削除 (DELETE /auth/account)

ユーザーアカウントを完全に削除します。

**リクエスト例:**
```bash
curl -X DELETE http://localhost:3000/auth/account \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "password": "CurrentPass123!",
    "confirmation": "DELETE"
  }'
```

## 認証フロー

### 基本的な認証フロー

1. **ユーザー登録** (`POST /auth/signup`)
2. **ログイン** (`POST /auth/signin`) → アクセストークン・リフレッシュトークン取得
3. **API使用** - アクセストークンをAuthorizationヘッダーで送信
4. **トークン更新** (`POST /auth/refresh`) - アクセストークン期限切れ時
5. **ログアウト** (`POST /auth/signout`)

### パスワードリセットフロー

1. **リセット要求** (`POST /auth/forgot-password`) → メール送信
2. **パスワード変更** (`POST /auth/reset-password`) - メール内のトークン使用

## セキュリティ仕様

- **アクセストークン**: 15分で自動期限切れ
- **リフレッシュトークン**: 7日間、自動更新対応
- **パスワードリセットトークン**: 1時間・使い切り
- **パスワードハッシュ**: Argon2 + 自動リハッシュ対応
- **認証方式**: Authorization ヘッダー または httpOnly クッキー