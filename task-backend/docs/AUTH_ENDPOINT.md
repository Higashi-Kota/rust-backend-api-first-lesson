# 認証エンドポイント確認ガイド

本ドキュメントでは、JWT認証フローに関するすべてのエンドポイントの確認方法を詳しく説明します。

## 🔐 認証フロー概要

- **アクセストークン**: 15分の短期間、JWTとしてhttpOnlyクッキーまたはAuthorizationヘッダーで送信
- **リフレッシュトークン**: 7日間、データベースで管理、自動ローテーション実装
- **パスワード**: Argon2でハッシュ化
- **CSRF保護**: SameSite cookieとCSRFトークン
- **セキュリティヘッダー**: 各レスポンスに自動付与

## 📋 認証エンドポイント一覧

### 1. ユーザー登録 (POST /auth/signup)

新規ユーザーアカウントの作成。

```bash
curl -X POST http://localhost:3000/auth/signup \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "SecurePass123!",
    "password_confirmation": "SecurePass123!"
  }' | jq
```

#### 成功レスポンス例 (201 Created):
```json
{
  "message": "User registered successfully",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "testuser",
    "email": "test@example.com",
    "email_verified": false,
    "is_active": true,
    "created_at": "2025-06-12T10:00:00Z",
    "updated_at": "2025-06-12T10:00:00Z"
  },
  "tokens": {
    "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
    "refresh_token": "550e8400-e29b-41d4-a716-446655440001",
    "access_token_expires_in": 900,
    "refresh_token_expires_in": 604800,
    "token_type": "Bearer"
  }
}
```

#### バリデーションエラー例 (400 Bad Request):
```bash
curl -X POST http://localhost:3000/auth/signup \
  -H "Content-Type: application/json" \
  -d '{
    "username": "a",
    "email": "invalid-email",
    "password": "123",
    "password_confirmation": "321"
  }' | jq
```

レスポンス:
```json
{
  "errors": [
    "username: Username must be at least 3 characters long",
    "email: Invalid email format",
    "password: Password must be at least 8 characters long",
    "password_confirmation: Password confirmation does not match"
  ],
  "error_type": "validation_errors"
}
```

### 2. ログイン (POST /auth/signin)

既存ユーザーの認証。

```bash
curl -X POST http://localhost:3000/auth/signin \
  -H "Content-Type: application/json" \
  -d '{
    "identifier": "test@example.com",
    "password": "SecurePass123!"
  }' | jq
```

#### 成功レスポンス例 (200 OK):
```json
{
  "message": "Successfully signed in",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "testuser",
    "email": "test@example.com",
    "email_verified": false,
    "is_active": true,
    "created_at": "2025-06-12T10:00:00Z",
    "updated_at": "2025-06-12T10:00:00Z"
  },
  "tokens": {
    "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
    "refresh_token": "550e8400-e29b-41d4-a716-446655440002",
    "access_token_expires_in": 900,
    "refresh_token_expires_in": 604800,
    "token_type": "Bearer"
  }
}
```

#### 認証失敗例 (401 Unauthorized):
```bash
curl -X POST http://localhost:3000/auth/signin \
  -H "Content-Type: application/json" \
  -d '{
    "identifier": "test@example.com",
    "password": "WrongPassword"
  }' | jq
```

レスポンス:
```json
{
  "error": "Invalid credentials",
  "error_type": "unauthorized"
}
```

### 3. トークンリフレッシュ (POST /auth/refresh)

期限切れのアクセストークンを新しいトークンに更新。

```bash
curl -X POST http://localhost:3000/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "550e8400-e29b-41d4-a716-446655440002"
  }' | jq
```

#### 成功レスポンス例 (200 OK):
```json
{
  "message": "Token refreshed successfully",
  "tokens": {
    "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
    "refresh_token": "550e8400-e29b-41d4-a716-446655440003",
    "access_token_expires_in": 900,
    "refresh_token_expires_in": 604800,
    "token_type": "Bearer"
  }
}
```

#### 無効なリフレッシュトークン (401 Unauthorized):
```bash
curl -X POST http://localhost:3000/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "invalid-token"
  }' | jq
```

### 4. 現在のユーザー情報取得 (GET /auth/me)

認証済みユーザーの情報を取得。

```bash
curl -X GET http://localhost:3000/auth/me \
  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..." | jq
```

#### 成功レスポンス例 (200 OK):
```json
{
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "testuser",
    "email": "test@example.com",
    "email_verified": false,
    "is_active": true,
    "created_at": "2025-06-12T10:00:00Z",
    "updated_at": "2025-06-12T10:00:00Z"
  }
}
```

### 5. ログアウト (POST /auth/signout)

現在のセッションを終了し、リフレッシュトークンを無効化。

```bash
curl -X POST http://localhost:3000/auth/signout \
  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..." \
  -H "Cookie: refresh_token=550e8400-e29b-41d4-a716-446655440003" | jq
```

#### 成功レスポンス例 (200 OK):
```json
{
  "message": "Successfully signed out"
}
```

### 6. 全デバイスからログアウト (POST /auth/signout-all)

すべてのデバイスからのログアウト（全リフレッシュトークンを無効化）。

```bash
curl -X POST http://localhost:3000/auth/signout-all \
  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..." | jq
```

#### 成功レスポンス例 (200 OK):
```json
{
  "message": "Successfully signed out from all devices"
}
```

### 7. パスワードリセット要求 (POST /auth/forgot-password)

パスワードリセットメールの送信要求。

```bash
curl -X POST http://localhost:3000/auth/forgot-password \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com"
  }' | jq
```

#### 成功レスポンス例 (200 OK):
```json
{
  "message": "If the email address is registered, you will receive a password reset email shortly."
}
```

### 8. パスワードリセット実行 (POST /auth/reset-password)

リセットトークンを使用してパスワードを変更。

```bash
curl -X POST http://localhost:3000/auth/reset-password \
  -H "Content-Type: application/json" \
  -d '{
    "token": "reset-token-from-email",
    "new_password": "NewSecurePass123!",
    "new_password_confirmation": "NewSecurePass123!"
  }' | jq
```

#### 成功レスポンス例 (200 OK):
```json
{
  "message": "Password has been reset successfully"
}
```

### 9. パスワード変更 (PUT /auth/change-password)

認証済みユーザーのパスワード変更。

```bash
curl -X PUT http://localhost:3000/auth/change-password \
  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "current_password": "SecurePass123!",
    "new_password": "NewSecurePass456!",
    "new_password_confirmation": "NewSecurePass456!"
  }' | jq
```

#### 成功レスポンス例 (200 OK):
```json
{
  "message": "Password changed successfully"
}
```

### 10. アカウント削除 (DELETE /auth/account)

認証済みユーザーのアカウント削除。

```bash
curl -X DELETE http://localhost:3000/auth/account \
  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "password": "SecurePass123!",
    "confirmation": "DELETE"
  }' | jq
```

#### 成功レスポンス例 (200 OK):
```json
{
  "message": "Account deleted successfully"
}
```

### 11. 認証ステータス確認 (GET /auth/status)

認証状態の確認（認証不要）。

```bash
curl -X GET http://localhost:3000/auth/status | jq
```

#### レスポンス例 (200 OK):
```json
{
  "authenticated": false,
  "user": null,
  "access_token_expires_in": null
}
```

## 🔒 認証フロー実例

### 完全な認証フローの例

#### 1. ユーザー登録
```bash
# 新規ユーザー登録
SIGNUP_RESPONSE=$(curl -s -X POST http://localhost:3000/auth/signup \
  -H "Content-Type: application/json" \
  -d '{
    "username": "newuser",
    "email": "newuser@example.com",
    "password": "SecurePass123!",
    "password_confirmation": "SecurePass123!"
  }')

# アクセストークンを抽出
ACCESS_TOKEN=$(echo $SIGNUP_RESPONSE | jq -r '.tokens.access_token')
REFRESH_TOKEN=$(echo $SIGNUP_RESPONSE | jq -r '.tokens.refresh_token')

echo "Access Token: $ACCESS_TOKEN"
echo "Refresh Token: $REFRESH_TOKEN"
```

#### 2. 認証が必要なリソースへのアクセス
```bash
# 現在のユーザー情報を取得
curl -X GET http://localhost:3000/auth/me \
  -H "Authorization: Bearer $ACCESS_TOKEN" | jq
```

#### 3. トークンリフレッシュ
```bash
# 15分後にアクセストークンが期限切れになった場合
REFRESH_RESPONSE=$(curl -s -X POST http://localhost:3000/auth/refresh \
  -H "Content-Type: application/json" \
  -d "{\"refresh_token\": \"$REFRESH_TOKEN\"}")

# 新しいトークンを取得
NEW_ACCESS_TOKEN=$(echo $REFRESH_RESPONSE | jq -r '.tokens.access_token')
NEW_REFRESH_TOKEN=$(echo $REFRESH_RESPONSE | jq -r '.tokens.refresh_token')
```

#### 4. セッション終了
```bash
# ログアウト
curl -X POST http://localhost:3000/auth/signout \
  -H "Authorization: Bearer $NEW_ACCESS_TOKEN" \
  -H "Cookie: refresh_token=$NEW_REFRESH_TOKEN" | jq
```

## 🛡️ セキュリティテスト

### 1. 無効なトークンでのアクセス試行
```bash
curl -X GET http://localhost:3000/auth/me \
  -H "Authorization: Bearer invalid_token" | jq
```

### 2. 期限切れトークンの確認
```bash
# 期限切れのトークンを使用してアクセス試行
curl -X GET http://localhost:3000/auth/me \
  -H "Authorization: Bearer expired_token_here" | jq
```

### 3. CSRFトークンの確認
```bash
# Cookieベースの認証でCSRF保護を確認
curl -X POST http://localhost:3000/auth/signout \
  -H "Cookie: access_token=valid_token_here" \
  -H "X-CSRF-Token: invalid_csrf_token" | jq
```

## 🔧 トラブルシューティング

### よくあるエラーと対処法

#### 1. 401 Unauthorized
- トークンが無効または期限切れ
- リフレッシュトークンを使用して新しいアクセストークンを取得

#### 2. 403 Forbidden
- ユーザーがリソースへのアクセス権限を持っていない
- 適切な権限を持つユーザーでログインし直す

#### 3. 422 Unprocessable Entity
- リクエストボディのバリデーションエラー
- 必須フィールドや形式を確認

#### 4. 429 Too Many Requests
- レート制限に達している
- しばらく待ってからリトライ

## 📝 注意事項

1. **本番環境では必ずHTTPS**を使用してください
2. **アクセストークンは短期間**（15分）で期限切れとなります
3. **リフレッシュトークンは自動ローテーション**されます
4. **パスワードはArgon2**でハッシュ化されています
5. **セキュリティヘッダー**が全レスポンスに付与されます
6. **レート制限**が認証エンドポイントに適用されています