# パブリックエンドポイント確認ガイド

本ドキュメントでは、認証不要で使用できるパブリックAPIエンドポイントの確認方法を詳しく説明します。

## 🌐 パブリックエンドポイントについて

これらのエンドポイントは認証なしでアクセス可能ですが、一部にはレート制限が適用されています。

## 🏠 基本エンドポイント

### 1. ルートエンドポイント (GET /)

APIの基本情報を確認。

```bash
curl -X GET http://localhost:3000/ | awk 4
```

#### 成功レスポンス例 (200 OK):
```
Task Backend API v1.0
```

### 2. ヘルスチェック (GET /health)

サーバーの稼働状況を確認。

```bash
curl -X GET http://localhost:3000/health | awk 4
```

#### 成功レスポンス例 (200 OK):
```
OK
```

## 🔓 認証関連パブリックエンドポイント

### 3. ユーザー登録 (POST /auth/signup)

新規ユーザーアカウントの作成。

```bash
curl -X POST http://localhost:3000/auth/signup \
  -H "Content-Type: application/json" \
  -d '{
    "username": "newuser",
    "email": "newuser@example.com",
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
    "username": "newuser",
    "email": "newuser@example.com",
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

#### ユーザー名重複エラー例 (409 Conflict):
```bash
curl -X POST http://localhost:3000/auth/signup \
  -H "Content-Type: application/json" \
  -d '{
    "username": "existinguser",
    "email": "existing@example.com",
    "password": "SecurePass123!",
    "password_confirmation": "SecurePass123!"
  }' | jq
```

レスポンス:
```json
{
  "error": "Username or email already exists",
  "error_type": "conflict"
}
```

### 4. ログイン (POST /auth/signin)

既存ユーザーの認証。

```bash
curl -X POST http://localhost:3000/auth/signin \
  -H "Content-Type: application/json" \
  -d '{
    "identifier": "newuser@example.com",
    "password": "SecurePass123!"
  }' | jq
```

#### 成功レスポンス例 (200 OK):
```json
{
  "message": "Successfully signed in",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "newuser",
    "email": "newuser@example.com",
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
    "identifier": "newuser@example.com",
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

#### アカウント無効エラー例 (403 Forbidden):
```bash
curl -X POST http://localhost:3000/auth/signin \
  -H "Content-Type: application/json" \
  -d '{
    "identifier": "inactive@example.com",
    "password": "SecurePass123!"
  }' | jq
```

レスポンス:
```json
{
  "error": "Account is inactive",
  "error_type": "forbidden"
}
```

### 5. トークンリフレッシュ (POST /auth/refresh)

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

レスポンス:
```json
{
  "error": "Invalid or expired refresh token",
  "error_type": "unauthorized"
}
```

### 6. パスワードリセット要求 (POST /auth/forgot-password)

パスワードリセットメールの送信要求。

```bash
curl -X POST http://localhost:3000/auth/forgot-password \
  -H "Content-Type: application/json" \
  -d '{
    "email": "newuser@example.com"
  }' | jq
```

#### 成功レスポンス例 (200 OK):
```json
{
  "message": "If the email address is registered, you will receive a password reset email shortly."
}
```

#### バリデーションエラー例 (400 Bad Request):
```bash
curl -X POST http://localhost:3000/auth/forgot-password \
  -H "Content-Type: application/json" \
  -d '{
    "email": "invalid-email"
  }' | jq
```

レスポンス:
```json
{
  "errors": [
    "email: Invalid email format"
  ],
  "error_type": "validation_errors"
}
```

### 7. パスワードリセット実行 (POST /auth/reset-password)

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

#### 無効なトークンエラー例 (400 Bad Request):
```bash
curl -X POST http://localhost:3000/auth/reset-password \
  -H "Content-Type: application/json" \
  -d '{
    "token": "invalid-token",
    "new_password": "NewSecurePass123!",
    "new_password_confirmation": "NewSecurePass123!"
  }' | jq
```

レスポンス:
```json
{
  "error": "Invalid or expired reset token",
  "error_type": "bad_request"
}
```

### 8. 認証ステータス確認 (GET /auth/status)

認証状態の確認（常に未認証として返される）。

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

## 🔄 パブリックエンドポイント使用例

### 新規ユーザー登録からログインまでの流れ

```bash
# 1. 新規ユーザー登録
echo "=== ユーザー登録 ==="
SIGNUP_RESPONSE=$(curl -s -X POST http://localhost:3000/auth/signup \
  -H "Content-Type: application/json" \
  -d '{
    "username": "demouser",
    "email": "demo@example.com",
    "password": "DemoPass123!",
    "password_confirmation": "DemoPass123!"
  }')

echo $SIGNUP_RESPONSE | jq

# 2. 登録後に自動発行されたトークンを取得
ACCESS_TOKEN=$(echo $SIGNUP_RESPONSE | jq -r '.tokens.access_token')
REFRESH_TOKEN=$(echo $SIGNUP_RESPONSE | jq -r '.tokens.refresh_token')

echo "Access Token: $ACCESS_TOKEN"
echo "Refresh Token: $REFRESH_TOKEN"

# 3. サーバーヘルスチェック
echo "=== ヘルスチェック ==="
curl -s -X GET http://localhost:3000/health

# 4. 認証ステータス確認
echo "=== 認証ステータス ==="
curl -s -X GET http://localhost:3000/auth/status | jq
```

### パスワードリセットの流れ

```bash
# 1. パスワードリセット要求
echo "=== パスワードリセット要求 ==="
curl -X POST http://localhost:3000/auth/forgot-password \
  -H "Content-Type: application/json" \
  -d '{
    "email": "demo@example.com"
  }' | jq

# 2. メールで受信したトークンを使用してパスワードリセット
# （実際のトークンはメールから取得する必要があります）
echo "=== パスワードリセット実行 ==="
curl -X POST http://localhost:3000/auth/reset-password \
  -H "Content-Type: application/json" \
  -d '{
    "token": "actual-reset-token-from-email",
    "new_password": "NewDemoPass123!",
    "new_password_confirmation": "NewDemoPass123!"
  }' | jq
```

### トークンリフレッシュの流れ

```bash
# アクセストークンが期限切れになった場合のリフレッシュ
echo "=== トークンリフレッシュ ==="
REFRESH_RESPONSE=$(curl -s -X POST http://localhost:3000/auth/refresh \
  -H "Content-Type: application/json" \
  -d "{\"refresh_token\": \"$REFRESH_TOKEN\"}")

echo $REFRESH_RESPONSE | jq

# 新しいトークンを取得
NEW_ACCESS_TOKEN=$(echo $REFRESH_RESPONSE | jq -r '.tokens.access_token')
NEW_REFRESH_TOKEN=$(echo $REFRESH_RESPONSE | jq -r '.tokens.refresh_token')

echo "New Access Token: $NEW_ACCESS_TOKEN"
echo "New Refresh Token: $NEW_REFRESH_TOKEN"
```

## ⚠️ エラーハンドリング

### よくあるエラーレスポンス

#### 1. バリデーションエラー (400 Bad Request)
```json
{
  "errors": [
    "Username must be at least 3 characters long",
    "Password must contain at least one uppercase letter"
  ],
  "error_type": "validation_errors"
}
```

#### 2. 認証失敗 (401 Unauthorized)
```json
{
  "error": "Invalid credentials",
  "error_type": "unauthorized"
}
```

#### 3. リソース競合 (409 Conflict)
```json
{
  "error": "Username or email already exists",
  "error_type": "conflict"
}
```

#### 4. レート制限 (429 Too Many Requests)
```json
{
  "error": "Rate limit exceeded. Please try again later.",
  "error_type": "rate_limit_exceeded"
}
```

#### 5. サーバーエラー (500 Internal Server Error)
```json
{
  "error": "Internal server error",
  "error_type": "internal_server_error"
}
```

## 🔒 セキュリティ考慮事項

### レート制限

認証関連のエンドポイントには以下のレート制限が適用されています：

- **ユーザー登録**: 1時間に5回まで（同一IPアドレスから）
- **ログイン**: 1時間に10回まで（同一IPアドレスから）
- **パスワードリセット**: 1時間に3回まで（同一メールアドレス）

### セキュリティヘッダー

すべてのレスポンスには以下のセキュリティヘッダーが自動的に付与されます：

```
Content-Security-Policy: default-src 'self'
X-Frame-Options: DENY
X-Content-Type-Options: nosniff
Referrer-Policy: strict-origin-when-cross-origin
Permissions-Policy: geolocation=(), microphone=(), camera=()
```

### パスワード要件

新しいパスワードは以下の要件を満たす必要があります：

- 最低8文字以上
- 大文字と小文字を含む
- 数字を含む
- 特殊文字を含む（推奨）

## 📊 レスポンス時間測定

### パフォーマンステスト例

```bash
# 各エンドポイントのレスポンス時間を測定
echo "=== レスポンス時間測定 ==="

echo "ヘルスチェック:"
time curl -s -o /dev/null http://localhost:3000/health

echo "認証ステータス:"
time curl -s -o /dev/null http://localhost:3000/auth/status

echo "ユーザー登録:"
time curl -s -o /dev/null -X POST http://localhost:3000/auth/signup \
  -H "Content-Type: application/json" \
  -d '{
    "username": "perftest",
    "email": "perftest@example.com",
    "password": "PerfTest123!",
    "password_confirmation": "PerfTest123!"
  }'
```

## 📝 注意事項

1. **HTTPS接続**: 本番環境では必ずHTTPS接続を使用してください
2. **レート制限**: 認証エンドポイントには厳格なレート制限があります
3. **パスワード強度**: 強力なパスワードを使用してください
4. **エラーハンドリング**: 適切なエラーハンドリングを実装してください
5. **ログ監視**: 異常なアクセスパターンを監視してください
6. **セキュリティアップデート**: 定期的にセキュリティアップデートを適用してください