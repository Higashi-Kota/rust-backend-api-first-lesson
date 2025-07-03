# 権限管理ベストプラクティス

## 概要

このドキュメントでは、動的パーミッションシステムを採用したRust製バックエンドAPIにおける権限管理のベストプラクティスをまとめています。

## 権限管理の階層

### 1. 権限の3つの軸

```rust
pub struct PermissionContext {
    // 1. ロール（静的権限）
    pub role: UserRole,
    
    // 2. サブスクリプション階層（機能制限）
    pub subscription_tier: SubscriptionTier,
    
    // 3. アクセススコープ（データ範囲）
    pub access_scope: AccessScope,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UserRole {
    Admin,
    Member,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SubscriptionTier {
    Free,
    Pro,
    Enterprise,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AccessScope {
    Own,         // 自分のデータのみ
    Team,        // チーム内のデータ
    Organization, // 組織内のデータ
    Global,      // 全データ（管理者のみ）
}
```

### 2. 権限チェックの実装パターン

```rust
pub struct PermissionService {
    user_repo: Arc<dyn UserRepository>,
    org_repo: Arc<dyn OrganizationRepository>,
    team_repo: Arc<dyn TeamRepository>,
}

impl PermissionService {
    /// リソースへのアクセス権限をチェック
    pub async fn check_resource_access(
        &self,
        user_id: Uuid,
        resource_type: ResourceType,
        resource_id: Uuid,
        required_permission: Permission,
    ) -> Result<(), PermissionError> {
        // ユーザー情報取得
        let user = self.user_repo
            .find_by_id(user_id)
            .await?
            .ok_or(PermissionError::UserNotFound)?;

        // 管理者は全アクセス可能
        if user.role == UserRole::Admin {
            return Ok(());
        }

        // リソースタイプに応じた権限チェック
        match resource_type {
            ResourceType::Task => self.check_task_permission(user_id, resource_id, required_permission).await,
            ResourceType::Team => self.check_team_permission(user_id, resource_id, required_permission).await,
            ResourceType::Organization => self.check_organization_permission(user_id, resource_id, required_permission).await,
        }
    }

    /// タスクへのアクセス権限チェック
    async fn check_task_permission(
        &self,
        user_id: Uuid,
        task_id: Uuid,
        permission: Permission,
    ) -> Result<(), PermissionError> {
        let task = self.task_repo
            .find_by_id(task_id)
            .await?
            .ok_or(PermissionError::ResourceNotFound)?;

        // 所有者チェック
        if task.user_id == user_id {
            return Ok(());
        }

        // チームメンバーチェック
        if let Some(team_id) = task.team_id {
            if self.is_team_member(user_id, team_id).await? {
                // 読み取りは許可、書き込みは権限による
                match permission {
                    Permission::Read => return Ok(()),
                    Permission::Write | Permission::Delete => {
                        let member = self.team_repo
                            .find_member(team_id, user_id)
                            .await?
                            .ok_or(PermissionError::InsufficientPermission)?;
                        
                        if member.role.can_manage_tasks() {
                            return Ok(());
                        }
                    }
                }
            }
        }

        Err(PermissionError::InsufficientPermission)
    }
}
```

## ミドルウェアでの権限チェック

### 1. 認証ミドルウェア

```rust
pub async fn auth_middleware<B>(
    State(state): State<AppState>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    // Authorizationヘッダーからトークン取得
    let token = extract_token(&req)?;
    
    // トークン検証
    let claims = jwt_service::verify_token(&token)?;
    
    // ユーザー情報をリクエストに追加
    req.extensions_mut().insert(AuthenticatedUser {
        user_id: claims.sub,
        role: claims.role,
        subscription_tier: claims.subscription_tier,
    });
    
    Ok(next.run(req).await)
}
```

### 2. 役割ベースアクセス制御（RBAC）

```rust
pub fn require_role(required_role: UserRole) -> impl Fn(AuthenticatedUser) -> Result<(), AppError> {
    move |user: AuthenticatedUser| {
        if user.role >= required_role {
            Ok(())
        } else {
            Err(AppError::Forbidden(format!(
                "This endpoint requires {} role",
                required_role
            )))
        }
    }
}

// 使用例
pub fn admin_routes() -> Router {
    Router::new()
        .route("/admin/users", get(list_all_users))
        .layer(middleware::from_fn(require_role(UserRole::Admin)))
}
```

### 3. サブスクリプションベース制限

```rust
pub fn require_subscription(min_tier: SubscriptionTier) -> impl Fn(AuthenticatedUser) -> Result<(), AppError> {
    move |user: AuthenticatedUser| {
        if user.subscription_tier >= min_tier {
            Ok(())
        } else {
            Err(AppError::SubscriptionRequired(format!(
                "This feature requires {} subscription or higher",
                min_tier
            )))
        }
    }
}

// 使用例
pub fn premium_routes() -> Router {
    Router::new()
        .route("/analytics/advanced", get(advanced_analytics))
        .layer(middleware::from_fn(require_subscription(SubscriptionTier::Pro)))
}
```

## 組織・チーム階層での権限管理

### 1. 組織権限マトリックス

```rust
#[derive(Debug, Clone)]
pub struct OrganizationPermissions {
    pub role: OrganizationRole,
    pub permissions: HashSet<OrganizationPermission>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OrganizationRole {
    Owner,    // 全権限
    Admin,    // 管理権限
    Member,   // 一般メンバー
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OrganizationPermission {
    ManageOrganization,
    ManageMembers,
    ManageTeams,
    ManageSubscription,
    ViewAnalytics,
    ExportData,
}

impl OrganizationRole {
    pub fn default_permissions(&self) -> HashSet<OrganizationPermission> {
        use OrganizationPermission::*;
        
        match self {
            OrganizationRole::Owner => {
                vec![
                    ManageOrganization,
                    ManageMembers,
                    ManageTeams,
                    ManageSubscription,
                    ViewAnalytics,
                    ExportData,
                ].into_iter().collect()
            }
            OrganizationRole::Admin => {
                vec![
                    ManageMembers,
                    ManageTeams,
                    ViewAnalytics,
                    ExportData,
                ].into_iter().collect()
            }
            OrganizationRole::Member => {
                vec![ViewAnalytics].into_iter().collect()
            }
        }
    }
}
```

### 2. 権限の継承と委譲

```rust
pub struct PermissionInheritance {
    /// 組織 -> 部門 -> チーム -> ユーザー
    pub hierarchy: Vec<PermissionNode>,
}

pub struct PermissionNode {
    pub level: HierarchyLevel,
    pub id: Uuid,
    pub inherited_permissions: HashSet<Permission>,
    pub explicit_permissions: HashSet<Permission>,
}

impl PermissionService {
    /// 有効な権限を計算（継承を考慮）
    pub async fn calculate_effective_permissions(
        &self,
        user_id: Uuid,
        context: PermissionContext,
    ) -> Result<HashSet<Permission>, AppError> {
        let mut permissions = HashSet::new();
        
        // 1. ユーザー個人の権限
        let user_permissions = self.get_user_permissions(user_id).await?;
        permissions.extend(user_permissions);
        
        // 2. チーム経由の権限
        let team_permissions = self.get_team_permissions(user_id).await?;
        permissions.extend(team_permissions);
        
        // 3. 組織経由の権限
        let org_permissions = self.get_organization_permissions(user_id).await?;
        permissions.extend(org_permissions);
        
        // 4. サブスクリプションに基づく制限を適用
        let filtered_permissions = self.apply_subscription_limits(
            permissions,
            context.subscription_tier
        );
        
        Ok(filtered_permissions)
    }
}
```

## データアクセススコープの実装

### 1. スコープベースのクエリフィルタリング

```rust
pub trait ScopedRepository {
    async fn find_with_scope(
        &self,
        user_id: Uuid,
        scope: AccessScope,
        filters: QueryFilters,
    ) -> Result<Vec<Self::Model>, DbErr>;
}

impl ScopedRepository for TaskRepository {
    async fn find_with_scope(
        &self,
        user_id: Uuid,
        scope: AccessScope,
        filters: QueryFilters,
    ) -> Result<Vec<Task>, DbErr> {
        let mut query = Task::find();
        
        // スコープに基づくフィルタリング
        match scope {
            AccessScope::Own => {
                query = query.filter(task::Column::UserId.eq(user_id));
            }
            AccessScope::Team => {
                // ユーザーが所属するチームのタスク
                let team_ids = self.get_user_team_ids(user_id).await?;
                query = query.filter(task::Column::TeamId.is_in(team_ids));
            }
            AccessScope::Organization => {
                // ユーザーが所属する組織のタスク
                let org_ids = self.get_user_organization_ids(user_id).await?;
                query = query.filter(task::Column::OrganizationId.is_in(org_ids));
            }
            AccessScope::Global => {
                // フィルタリングなし（管理者のみ）
            }
        }
        
        // 追加フィルターを適用
        query = apply_filters(query, filters);
        
        query.all(&self.db).await
    }
}
```

### 2. Row-Level Security (RLS) の実装

```rust
pub struct RLSPolicy {
    pub resource_type: ResourceType,
    pub rules: Vec<AccessRule>,
}

pub struct AccessRule {
    pub condition: AccessCondition,
    pub allowed_operations: Vec<Operation>,
}

pub enum AccessCondition {
    IsOwner,
    IsTeamMember { role: Option<TeamRole> },
    IsOrganizationMember { role: Option<OrganizationRole> },
    HasPermission(Permission),
    Custom(Box<dyn Fn(&User, &Resource) -> bool>),
}

impl RLSPolicy {
    pub fn evaluate(&self, user: &User, resource: &Resource, operation: Operation) -> bool {
        self.rules.iter().any(|rule| {
            rule.allowed_operations.contains(&operation) &&
            self.check_condition(&rule.condition, user, resource)
        })
    }
    
    fn check_condition(&self, condition: &AccessCondition, user: &User, resource: &Resource) -> bool {
        match condition {
            AccessCondition::IsOwner => resource.owner_id == user.id,
            AccessCondition::IsTeamMember { role } => {
                // チームメンバーシップチェック
                self.check_team_membership(user.id, resource.team_id, role)
            }
            // ... 他の条件
        }
    }
}
```

## セキュリティベストプラクティス

### 1. 最小権限の原則

```rust
// デフォルトで拒否
pub fn default_deny<T>() -> Result<T, AppError> {
    Err(AppError::Forbidden("Access denied by default".to_string()))
}

// 明示的な許可が必要
pub async fn check_permission_explicit(
    user_id: Uuid,
    resource_id: Uuid,
    required_permission: Permission,
) -> Result<(), AppError> {
    let has_permission = permission_service
        .has_explicit_permission(user_id, resource_id, required_permission)
        .await?;
    
    if has_permission {
        Ok(())
    } else {
        default_deny()
    }
}
```

### 2. 権限昇格の防止

```rust
pub struct ElevationGuard {
    original_permissions: HashSet<Permission>,
    elevated: bool,
}

impl ElevationGuard {
    pub fn elevate_temporarily(
        &mut self,
        additional_permissions: HashSet<Permission>,
    ) -> Result<(), AppError> {
        if self.elevated {
            return Err(AppError::InternalServerError(
                "Permission elevation already in progress".to_string()
            ));
        }
        
        self.original_permissions.extend(additional_permissions);
        self.elevated = true;
        Ok(())
    }
}

impl Drop for ElevationGuard {
    fn drop(&mut self) {
        // 権限を元に戻す
        self.elevated = false;
    }
}
```

### 3. 監査ログ

```rust
#[derive(Debug, Serialize)]
pub struct PermissionAuditLog {
    pub timestamp: DateTime<Utc>,
    pub user_id: Uuid,
    pub resource_type: ResourceType,
    pub resource_id: Uuid,
    pub action: String,
    pub granted: bool,
    pub reason: Option<String>,
}

pub async fn audit_permission_check(
    user_id: Uuid,
    resource: &Resource,
    action: &str,
    result: &Result<(), PermissionError>,
) {
    let log = PermissionAuditLog {
        timestamp: Utc::now(),
        user_id,
        resource_type: resource.resource_type(),
        resource_id: resource.id(),
        action: action.to_string(),
        granted: result.is_ok(),
        reason: result.as_ref().err().map(|e| e.to_string()),
    };
    
    // 非同期でログを記録
    tokio::spawn(async move {
        if let Err(e) = audit_service::log(log).await {
            tracing::error!("Failed to log permission audit: {}", e);
        }
    });
}
```

## まとめ

効果的な権限管理は、セキュアで柔軟なAPIの基盤です。役割、サブスクリプション、スコープの3軸で権限を管理し、最小権限の原則に従い、すべてのアクセスを監査することで、エンタープライズグレードの権限管理を実現できます。