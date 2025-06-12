現代の認証認可における JWT におけるベストプラクティスに従ったものでお願いいたします。
新規に作成するファイルないしは修正が必要なファイルは修正内容を含む全量を省略せずに提案してください。
1 ファイル出力するたびに確認を仰いでください。その時に、次に出力するファイルを提示して、完成までの残ステップを表示してください。

## 🔐 JWT 認証認可 API エンドポイント設計

### 認証関連エンドポイント

```
POST   /auth/signup          - ユーザー登録
POST   /auth/signin          - ログイン
POST   /auth/signout         - ログアウト
POST   /auth/refresh         - アクセストークンリフレッシュ
DELETE /auth/account         - アカウント削除
POST   /auth/forgot-password - パスワードリセット要求
POST   /auth/reset-password  - パスワードリセット実行
GET    /auth/me              - 現在のユーザー情報取得
```

### セキュリティベストプラクティス

- **アクセストークン**: 15 分の短期間、JWT として httpOnly クッキーで送信
- **リフレッシュトークン**: 7 日間、データベースで管理、ローテーション実装
- **パスワード**: Argon2 でハッシュ化
- **CSRF 保護**: SameSite cookie と CSRF トークン
- **レート制限**: 認証エンドポイントに適用
- **パスワードリセット**: 一時的なトークン（1 時間有効）

## 📁 作成・修正ファイル一覧

### 🆕 新規作成ファイル（20 個）

#### マイグレーション（4 個）

- [x] 1. `migration/src/m20250612_000001_create_users_table.rs`
- [x] 2. `migration/src/m20250612_000002_create_refresh_tokens_table.rs`
- [x] 3. `migration/src/m20250612_000003_create_password_reset_tokens_table.rs`
- [x] 4. `migration/src/m20250612_000004_add_user_id_to_tasks.rs`

#### ドメインモデル（3 個）

- [x] 5. `task-backend/src/domain/user_model.rs`
- [x] 6. `task-backend/src/domain/refresh_token_model.rs`
- [x] 7. `task-backend/src/domain/password_reset_token_model.rs`

#### リポジトリ（3 個）

- [x] 8. `task-backend/src/repository/user_repository.rs`
- [x] 9. `task-backend/src/repository/refresh_token_repository.rs`
- [x] 10. `task-backend/src/repository/password_reset_token_repository.rs`

#### サービス（2 個）

- [x] 11. `task-backend/src/service/auth_service.rs`
- [x] 12. `task-backend/src/service/user_service.rs`

#### DTO（2 個）

- [x] 13. `task-backend/src/api/dto/auth_dto.rs`
- [x] 14. `task-backend/src/api/dto/user_dto.rs`

#### ハンドラー（2 個）

- [x] 15. `task-backend/src/api/handlers/auth_handler.rs`
- [x] 16. `task-backend/src/api/handlers/user_handler.rs`

#### ユーティリティ（4 個）

- [x] 17. `task-backend/src/middleware/auth.rs`
- [x] 18. `task-backend/src/utils/jwt.rs`
- [x] 19. `task-backend/src/utils/password.rs`
- [x] 20. `task-backend/src/utils/email.rs`

### 🔧 修正ファイル（14 個）

- [x] 1. `Cargo.toml` - workspace 依存関係追加
- [x] 2. `task-backend/Cargo.toml` - 新しい依存関係追加
- [x] 3. `migration/src/lib.rs` - 新マイグレーション登録
- [x] 4. `task-backend/src/lib.rs` - 新モジュール追加
- [x] 5. `task-backend/src/domain/mod.rs` - 新モデル追加
- [x] 6. `task-backend/src/repository/mod.rs` - 新リポジトリ追加
- [x] 7. `task-backend/src/service/mod.rs` - 新サービス追加
- [x] 8. `task-backend/src/api/dto/mod.rs` - 新 DTO 追加
- [x] 9. `task-backend/src/api/handlers/mod.rs` - 新ハンドラー追加
- [x] 10. `task-backend/src/main.rs` - 認証ルーター統合
- [x] 11. `task-backend/src/api/handlers/task_handler.rs` - 認証ミドルウェア適用
- [x] 12. `task-backend/src/domain/task_model.rs` - user_id 追加
- [x] 13. `task-backend/src/repository/task_repository.rs` - ユーザーフィルタリング
- [x] 14. `task-backend/src/service/task_service.rs` - ユーザー関連処理
