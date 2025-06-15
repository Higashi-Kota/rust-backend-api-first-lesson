# 🛡️ 動的権限システムの設計

同一のエンドポイントであっても、ユーザーの**ロール**・**サブスクリプション**・**アクセススコープ**に応じて適切な振る舞いを切り替える柔軟な設計。

---

## 1. 🧱 権限・特権モデルの拡張

### 🎯 権限（Permission）

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub resource: String,      // e.g. "tasks", "users", "reports"
    pub action: String,        // e.g. "read", "write", "delete", "admin"
    pub scope: PermissionScope,
}
```

### 🌐 権限スコープ（PermissionScope）

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionScope {
    Own,           // 自分のデータのみ
    Team,          // チームのデータ
    Organization,  // 組織全体
    Global,        // 全データ
}
```

### 🎁 特権（Privilege）

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Privilege {
    pub name: String,                        // e.g. "bulk_export"
    pub subscription_tier: SubscriptionTier,
    pub quota: Option<PermissionQuota>,
}
```

### 📊 特権クォータ（PermissionQuota）

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionQuota {
    pub max_items: Option<u32>,       // 最大取得件数
    pub rate_limit: Option<u32>,      // レート制限
    pub features: Vec<String>,        // 利用可能機能
}
```

---

## 2. 💎 サブスクリプション階層

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubscriptionTier {
    Free,
    Pro,
    Enterprise,
}
```

### 🔍 権限チェック + 特権取得ロジック

```rust
impl RoleWithPermissions {
    pub fn can_perform_action(&self, resource: &str, action: &str, target_user_id: Option<Uuid>) -> PermissionResult {
        let base_permission = self.get_base_permission(resource, action);
        let subscription_privilege = self.get_subscription_privilege(resource, action);
        PermissionResult::new(base_permission, subscription_privilege, target_user_id)
    }

    pub fn get_subscription_privilege(&self, resource: &str, action: &str) -> Option<Privilege> {
        match (&self.subscription_tier, resource, action) {
            (SubscriptionTier::Free, "tasks", "read") => Some(Privilege {
                name: "basic_task_access".into(),
                subscription_tier: SubscriptionTier::Free,
                quota: Some(PermissionQuota {
                    max_items: Some(100),
                    rate_limit: Some(10),
                    features: vec!["basic_filter".into()],
                }),
            }),
            (SubscriptionTier::Pro, "tasks", "read") => Some(Privilege {
                name: "pro_task_access".into(),
                subscription_tier: SubscriptionTier::Pro,
                quota: Some(PermissionQuota {
                    max_items: Some(10_000),
                    rate_limit: Some(100),
                    features: vec!["advanced_filter".into(), "export".into()],
                }),
            }),
            (SubscriptionTier::Enterprise, "tasks", "read") => Some(Privilege {
                name: "enterprise_task_access".into(),
                subscription_tier: SubscriptionTier::Enterprise,
                quota: None,
                features: vec!["unlimited_access".into(), "bulk_operations".into()],
            }),
            _ => None,
        }
    }
}
```

---

## 3. 🧠 サービス層の動的切り替えロジック

```rust
impl TaskService {
    pub async fn list_tasks_dynamic(&self, user: &AuthenticatedUser, filter: Option<TaskFilterDto>) -> AppResult<TaskResponse> {
        let permission_result = user.0.can_perform_action("tasks", "read", None);

        match permission_result {
            PermissionResult::Allowed { privilege, scope } => {
                self.execute_task_query(user, filter, privilege, scope).await
            }
            PermissionResult::Denied { reason } => {
                Err(AppError::Forbidden(reason))
            }
        }
    }

    async fn execute_task_query(
        &self,
        user: &AuthenticatedUser,
        filter: Option<TaskFilterDto>,
        privilege: Option<Privilege>,
        scope: PermissionScope,
    ) -> AppResult<TaskResponse> {
        match (scope, privilege.as_ref()) {
            (PermissionScope::Own, Some(priv)) if priv.subscription_tier == SubscriptionTier::Free => {
                self.list_tasks_for_user_limited(user.0.user_id, priv.quota.as_ref()).await
            }

            (PermissionScope::Team, Some(priv)) if priv.subscription_tier == SubscriptionTier::Pro => {
                self.list_tasks_for_team_with_features(user.0.user_id, &priv.features, filter).await
            }

            (PermissionScope::Global, Some(priv)) if priv.subscription_tier == SubscriptionTier::Enterprise => {
                self.list_all_tasks_unlimited(filter).await
            }

            _ if user.0.is_admin() => {
                self.list_all_tasks_unlimited(filter).await
            }

            _ => {
                self.list_tasks_for_user(user.0.user_id)
                    .await
                    .map(TaskResponse::Limited)
            }
        }
    }
}
```

---

## 4. 🧩 エンドポイントでの透過的適用

```rust
pub async fn list_tasks_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Query(filter): Query<TaskFilterDto>,
) -> AppResult<Json<TaskResponse>> {
    let response = app_state
        .task_service
        .list_tasks_dynamic(&user, Some(filter))
        .await?;

    Ok(Json(response))
}
```

---

## 5. 🧪 具体的利用シナリオ

| ユーザー種別        | リクエスト例                                  | 呼び出される処理                         |
| ------------------- | --------------------------------------------- | ---------------------------------------- |
| Free ユーザー       | `GET /tasks?status=todo`                      | `list_tasks_for_user_limited(...)`       |
| Pro ユーザー        | `GET /tasks?status=todo&export=true`          | `list_tasks_for_team_with_features(...)` |
| Enterprise ユーザー | `GET /tasks?status=todo&bulk_operations=true` | `list_all_tasks_unlimited(...)`          |
| 管理者              | `GET /tasks`                                  | `list_all_tasks_unlimited(...)`          |

---

## ✅ 設計のメリット

- **単一の API で複数層のビジネスロジックをカプセル化**
- **権限・プランの追加が非破壊的**（定義追加だけで拡張可能）
- **セキュリティと柔軟性を両立**（型安全 + 実行時判定）
