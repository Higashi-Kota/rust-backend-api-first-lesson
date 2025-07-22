# 統一権限チェックミドルウェアガイド

## 概要

統一権限チェックミドルウェアは、Task Backend APIにおける権限管理を一元化し、コードの簡潔性と保守性を向上させるための仕組みです。本ガイドでは、統一権限チェックミドルウェアの設計思想、使用方法、移行手順について説明します。

## 目次

1. [基本概念](#基本概念)
2. [使用方法](#使用方法)
3. [APIリファレンス](#apiリファレンス)
4. [移行ガイド](#移行ガイド)
5. [実装例](#実装例)
6. [ベストプラクティス](#ベストプラクティス)
7. [トラブルシューティング](#トラブルシューティング)

## 基本概念

### Action（アクション）

リソースに対して実行可能な操作を定義します：

- `View`: リソースの閲覧
- `Create`: リソースの作成
- `Update`: リソースの更新
- `Delete`: リソースの削除
- `Admin`: 管理者専用操作

### Resources（リソース）

権限管理の対象となるリソースを定義します：

```rust
pub mod resources {
    pub const TASK: &str = "task";
    pub const TEAM: &str = "team";
    pub const USER: &str = "user";
    pub const ORGANIZATION: &str = "organization";
    pub const ROLE: &str = "role";
    pub const ANALYTICS: &str = "analytics";
    pub const PAYMENT: &str = "payment";
    pub const SUBSCRIPTION: &str = "subscription";
}
```

### 権限チェックの流れ

1. **認証チェック**: JWTトークンの検証
2. **ロール取得**: ユーザーのロール情報を取得
3. **リソース特定**: URLパスからリソースIDを抽出
4. **権限検証**: ロールとアクションの組み合わせを検証
5. **コンテキスト設定**: 権限情報をリクエストに付加

## 使用方法

### 1. 基本的な使用例

```rust
use crate::middleware::authorization::{resources, Action};
use crate::require_permission;

// タスク作成エンドポイントに権限チェックを適用
Router::new()
    .route(
        "/tasks",
        post(create_task)
            .route_layer(require_permission!(resources::TASK, Action::Create))
    )
```

### 2. 管理者専用エンドポイント

```rust
use crate::middleware::authorization::admin_permission_middleware;

// 管理者専用ルーター
Router::new()
    .route("/admin/users", get(list_all_users))
    .route("/admin/system/info", get(get_system_info))
    .layer(admin_permission_middleware())
```

### 3. リソース別の権限チェック

```rust
// チーム更新（チーム所有者のみ）
Router::new()
    .route(
        "/teams/{id}",
        patch(update_team)
            .route_layer(require_permission!(resources::TEAM, Action::Update))
    )

// ユーザー情報閲覧（本人または管理者）
Router::new()
    .route(
        "/users/{id}",
        get(get_user_details)
            .route_layer(require_permission!(resources::USER, Action::View))
    )
```

## APIリファレンス

### ミドルウェア関数

#### `admin_permission_middleware()`

管理者権限をチェックするミドルウェアを返します。

```rust
pub fn admin_permission_middleware() -> impl Fn(Request, Next) -> Pin<Box<dyn Future<Output = Result<Response, Response>> + Send>> + Clone + Send + 'static
```

**使用例:**
```rust
let admin_router = Router::new()
    .route("/admin/users", get(list_users))
    .layer(admin_permission_middleware());
```

#### `permission_middleware(resource, action)`

指定されたリソースとアクションに対する権限をチェックするミドルウェアを返します。

```rust
pub fn permission_middleware(
    resource: &'static str,
    action: Action,
) -> impl Fn(Request, Next) -> Pin<Box<dyn Future<Output = Result<Response, Response>> + Send>> + Clone + Send + 'static
```

**パラメータ:**
- `resource`: チェック対象のリソース名（`resources`モジュールの定数を使用）
- `action`: 実行しようとしているアクション

### マクロ

#### `require_permission!`

権限チェックミドルウェアを簡単に適用するためのマクロ。

```rust
require_permission!(resource, action)
```

**パラメータ:**
- `resource`: チェック対象のリソース（例: `resources::TASK`）
- `action`: 実行アクション（例: `Action::Create`）

**使用例:**
```rust
Router::new()
    .route(
        "/tasks",
        post(create_task)
            .route_layer(require_permission!(resources::TASK, Action::Create))
    )
    .route(
        "/tasks/{id}",
        delete(delete_task)
            .route_layer(require_permission!(resources::TASK, Action::Delete))
    )
```

### 型定義

#### `Action`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    View,    // リソースの閲覧
    Create,  // リソースの作成
    Update,  // リソースの更新
    Delete,  // リソースの削除
    Admin,   // 管理者操作
}
```

#### `RequirePermission`

```rust
#[derive(Clone)]
pub struct RequirePermission {
    pub resource: &'static str,
    pub action: Action,
}
```

#### `PermissionContext`

権限チェック後にリクエストに付加される権限情報。

```rust
#[derive(Clone, Debug)]
pub struct PermissionContext {
    pub user_id: Uuid,
    pub role: RoleWithPermissions,
    pub resource: &'static str,
    pub action: Action,
}
```

**メソッド:**
- `is_admin()`: ユーザーが管理者かどうかを確認
- `can_access()`: 現在のコンテキストでアクセス可能かを確認

### エラーレスポンス

#### 認証エラー（401 Unauthorized）
```json
{
    "error": {
        "code": "UNAUTHORIZED",
        "message": "Authentication required"
    }
}
```

#### 権限エラー（403 Forbidden）
```json
{
    "error": {
        "code": "FORBIDDEN",
        "message": "You don't have permission to perform this action"
    }
}
```

## 移行ガイド

### 既存コードからの移行

#### Before（直接的な権限チェック）
```rust
pub async fn create_task(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateTaskDto>,
) -> AppResult<impl IntoResponse> {
    // 個別の権限チェック
    app_state
        .permission_service
        .check_resource_access(user.user_id(), "task", None, "create")
        .await?;
    
    // ビジネスロジック
    // ...
}
```

#### After（統一権限チェック）
```rust
// ルーター定義時に権限チェックを設定
Router::new()
    .route(
        "/tasks",
        post(create_task)
            .route_layer(require_permission!(resources::TASK, Action::Create))
    )

// ハンドラーは権限チェック済みの前提で実装
pub async fn create_task(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateTaskDto>,
) -> AppResult<impl IntoResponse> {
    // ビジネスロジックのみ
    // ...
}
```

### 移行手順

1. **既存の権限チェックコードの特定**
   - `is_admin()`の直接呼び出し
   - `permission_service.check_resource_access()`の呼び出し
   - その他のカスタム権限チェック

2. **ルーター定義の更新**
   - 適切なミドルウェアをルートに適用
   - `require_permission!`マクロまたは`admin_permission_middleware()`を使用

3. **ハンドラーコードのクリーンアップ**
   - 権限チェックコードを削除
   - ビジネスロジックのみに集中

4. **テストの更新**
   - 権限チェックのテストケースを確認
   - 必要に応じてテストを調整

## 実装例

### 完全なルーター実装例

```rust
use crate::middleware::authorization::{
    admin_permission_middleware, permission_middleware, resources, Action
};
use crate::require_permission;

pub fn create_app_router(app_state: AppState) -> Router<AppState> {
    Router::new()
        // 公開エンドポイント（認証不要）
        .route("/health", get(health_check))
        
        // 認証が必要なエンドポイント
        .nest("/api", api_router(app_state.clone()))
        
        // 管理者専用エンドポイント
        .nest("/admin", admin_router(app_state.clone()))
}

fn api_router(app_state: AppState) -> Router<AppState> {
    Router::new()
        // タスク関連
        .route(
            "/tasks",
            get(list_tasks)  // 権限チェック不要（自分のタスクのみ表示）
                .post(create_task)
                .route_layer(require_permission!(resources::TASK, Action::Create))
        )
        .route(
            "/tasks/{id}",
            get(get_task)
                .route_layer(require_permission!(resources::TASK, Action::View))
                .patch(update_task)
                .route_layer(require_permission!(resources::TASK, Action::Update))
                .delete(delete_task)
                .route_layer(require_permission!(resources::TASK, Action::Delete))
        )
        
        // チーム関連
        .route(
            "/teams",
            get(list_teams)
                .post(create_team)
                .route_layer(require_permission!(resources::TEAM, Action::Create))
        )
        .route(
            "/teams/{id}",
            get(get_team)
                .route_layer(require_permission!(resources::TEAM, Action::View))
                .patch(update_team)
                .route_layer(require_permission!(resources::TEAM, Action::Update))
                .delete(delete_team)
                .route_layer(require_permission!(resources::TEAM, Action::Delete))
        )
        .with_state(app_state)
}

fn admin_router(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/users", get(list_all_users))
        .route("/system/info", get(get_system_info))
        .route("/analytics/summary", get(get_analytics_summary))
        // すべてのルートに管理者権限チェックを適用
        .layer(admin_permission_middleware())
        .with_state(app_state)
}
```

### カスタム権限チェックの実装

特殊な権限チェックが必要な場合の実装例：

```rust
/// チームメンバーのみがアクセス可能なエンドポイント
pub async fn get_team_tasks(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path(team_id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    // チームメンバーシップを確認
    let is_member = app_state
        .team_service
        .is_team_member(team_id, user.user_id())
        .await?;
    
    if !is_member && !user.is_admin() {
        return Err(AppError::Forbidden(
            "You must be a team member to access team tasks".to_string()
        ));
    }
    
    // タスク取得処理
    let tasks = app_state
        .task_service
        .get_team_tasks(team_id)
        .await?;
    
    Ok(Json(tasks))
}
```

### ハンドラー内での権限コンテキスト取得

```rust
pub async fn update_task(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Extension(permission_ctx): Extension<PermissionContext>,
    Path(task_id): Path<Uuid>,
    Json(payload): Json<UpdateTaskDto>,
) -> AppResult<impl IntoResponse> {
    // permission_ctxから追加の権限情報を取得可能
    if permission_ctx.is_admin() {
        // 管理者の場合の特別な処理
    }
    
    // 通常の処理
}
```

## ベストプラクティス

### 1. リソース名の統一

必ず`resources`モジュールの定数を使用してください：

```rust
// ✅ 良い例
require_permission!(resources::TASK, Action::Create)

// ❌ 悪い例
require_permission!("task", Action::Create)
```

### 2. アクションの明確化

各エンドポイントで必要な最小限の権限を設定：

```rust
// 読み取り専用のエンドポイント
.route_layer(require_permission!(resources::TASK, Action::View))

// 更新が必要なエンドポイント
.route_layer(require_permission!(resources::TASK, Action::Update))
```

### 3. エラーコンテキスト

エラー発生時の追跡のため、適切なコンテキストを設定：

```rust
// エラーハンドリングのベストプラクティス
internal_server_error(
    e,
    "task_handler::create_task",  // モジュール名::関数名
    "Failed to create task"        // ユーザー向けメッセージ
)
```

### 4. テストの実施

各権限パターンに対する統合テストを作成：

```rust
#[tokio::test]
async fn test_task_creation_permission() {
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;
    
    // タスク作成（成功）
    let response = app
        .clone()
        .oneshot(create_authenticated_request(
            "POST",
            "/tasks",
            &user.token,
            Some(json!({
                "title": "Test Task",
                "description": "Test Description"
            }))
        ))
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_admin_endpoint_access() {
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;
    let admin = authenticate_as_admin(&app).await;
    
    // 一般ユーザーのアクセス（失敗）
    let response = app
        .clone()
        .oneshot(create_authenticated_request(
            "GET",
            "/admin/users",
            &user.token,
            None
        ))
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    
    // 管理者のアクセス（成功）
    let response = app
        .clone()
        .oneshot(create_authenticated_request(
            "GET",
            "/admin/users",
            &admin.token,
            None
        ))
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}
```

## トラブルシューティング

### 権限チェックが期待通りに動作しない

1. **JWTトークンの確認**
   - トークンにロール情報が含まれているか
   - トークンの有効期限が切れていないか

2. **リソースIDの抽出**
   - URLパスから正しくリソースIDが抽出されているか
   - パスパラメータの形式が正しいか（`{id}`形式）

3. **ロール設定の確認**
   - ユーザーに適切なロールが割り当てられているか
   - ロールの権限設定が正しいか

### パフォーマンスの問題

1. **権限チェックのログ**
   - 処理時間を測定してボトルネックを特定
   - 不要な権限チェックの重複がないか確認

2. **キャッシュの活用**
   - ロール情報のキャッシュを検討
   - 頻繁にアクセスされるリソースの権限情報をキャッシュ

3. **データベースクエリ**
   - 権限チェックに関連するクエリの最適化
   - 適切なインデックスの設定

### エラーメッセージの改善

権限エラーが発生した場合の詳細な情報提供：

```rust
// 詳細なエラーメッセージ
Err(forbidden_error(
    &format!("User {} does not have {:?} permission for resource {}", 
        user_id, action, resource),
    &format!("authorization::check_permission::{}", resource),
    "You don't have permission to perform this action",
))
```

## パフォーマンス最適化

### 1. 早期リジェクト

ミドルウェアレベルで権限チェックを行うため、不正なリクエストを早期に拒否：

```rust
// ミドルウェアでの早期チェック
if !has_permission {
    return Err(forbidden_response());
}
// ハンドラーは実行されない
```

### 2. キャッシュ活用

ロール情報はJWTに含まれるため、追加のDB参照が不要：

```rust
// JWTからロール情報を直接取得
let role = user.claims.role;
// DBアクセス不要
```

### 3. 並列処理

権限チェックとビジネスロジックの分離により、処理の最適化が容易：

```rust
// 権限チェックは既に完了
// ビジネスロジックに集中できる
let result = service.process_request(payload).await?;
```

## 今後の拡張

### 動的権限（条件付き権限）のサポート

```rust
// 将来的な実装例
require_permission!(
    resources::TASK, 
    Action::Update,
    |user, resource| {
        // カスタム権限ロジック
        resource.owner_id == user.id || user.is_team_member(resource.team_id)
    }
)
```

### 権限の委譲機能

```rust
// 権限の一時的な委譲
delegate_permission!(
    from: user_a,
    to: user_b,
    resource: resources::TASK,
    action: Action::Update,
    expires_at: timestamp
)
```

### より細かい権限制御（フィールドレベル）

```rust
// フィールドレベルの権限
field_permission!(
    resources::TASK,
    fields: ["title", "description"],
    action: Action::Update
)
```

### 権限チェックの監査ログ強化

```rust
// 権限チェックの結果を記録
audit_log.record_permission_check(
    user_id,
    resource,
    action,
    result,
    timestamp
)
```

## まとめ

統一権限チェックミドルウェアにより、以下のメリットが得られます：

1. **コードの簡潔性**: ハンドラーからボイラープレートコードを削除
2. **保守性の向上**: 権限ロジックの一元管理
3. **セキュリティの強化**: 一貫した権限チェックの適用
4. **パフォーマンスの向上**: 早期リジェクトとキャッシュ活用
5. **テストの容易性**: 権限チェックとビジネスロジックの分離

本ガイドに従って実装することで、セキュアで保守しやすいAPIを構築できます。