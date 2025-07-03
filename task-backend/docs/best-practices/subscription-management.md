# サブスクリプション管理ベストプラクティス

## 概要

このドキュメントでは、SaaS型アプリケーションにおけるサブスクリプション管理のベストプラクティスをまとめています。

## サブスクリプションモデルの設計

### 1. 階層構造の定義

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SubscriptionTier {
    Free = 0,
    Pro = 1,
    Enterprise = 2,
}

impl SubscriptionTier {
    pub fn level(&self) -> u8 {
        *self as u8
    }
    
    pub fn features(&self) -> SubscriptionFeatures {
        match self {
            SubscriptionTier::Free => SubscriptionFeatures {
                max_tasks: 100,
                max_teams: 0,  // チーム機能なし
                max_team_members: 0,
                max_organizations: 0,
                rate_limit_per_minute: 10,
                data_export: false,
                advanced_analytics: false,
                priority_support: false,
            },
            SubscriptionTier::Pro => SubscriptionFeatures {
                max_tasks: 10_000,
                max_teams: 3,
                max_team_members: 10,
                max_organizations: 0,  // 組織機能なし
                rate_limit_per_minute: 100,
                data_export: true,
                advanced_analytics: true,
                priority_support: false,
            },
            SubscriptionTier::Enterprise => SubscriptionFeatures {
                max_tasks: u32::MAX,  // 無制限
                max_teams: 100,
                max_team_members: 1000,
                max_organizations: 10,
                rate_limit_per_minute: u32::MAX,  // 無制限
                data_export: true,
                advanced_analytics: true,
                priority_support: true,
            },
        }
    }
}
```

### 2. 機能フラグの実装

```rust
pub struct FeatureFlags {
    flags: HashMap<String, FeatureFlag>,
}

pub struct FeatureFlag {
    pub name: String,
    pub enabled: bool,
    pub minimum_tier: Option<SubscriptionTier>,
    pub rollout_percentage: Option<f32>,
}

impl FeatureFlags {
    pub fn is_enabled_for_user(
        &self,
        feature_name: &str,
        user: &User,
    ) -> bool {
        let Some(flag) = self.flags.get(feature_name) else {
            return false;
        };
        
        // 基本的な有効性チェック
        if !flag.enabled {
            return false;
        }
        
        // サブスクリプション階層チェック
        if let Some(min_tier) = flag.minimum_tier {
            if user.subscription_tier < min_tier {
                return false;
            }
        }
        
        // 段階的ロールアウトチェック
        if let Some(percentage) = flag.rollout_percentage {
            let user_hash = calculate_user_hash(&user.id);
            let threshold = (percentage * u32::MAX as f32) as u32;
            if user_hash > threshold {
                return false;
            }
        }
        
        true
    }
}
```

## サブスクリプション変更の処理

### 1. アップグレード処理

```rust
pub struct SubscriptionService {
    user_repo: Arc<dyn UserRepository>,
    subscription_history_repo: Arc<dyn SubscriptionHistoryRepository>,
    billing_service: Arc<dyn BillingService>,
}

impl SubscriptionService {
    pub async fn upgrade_subscription(
        &self,
        user_id: Uuid,
        new_tier: SubscriptionTier,
        payment_method: PaymentMethod,
    ) -> Result<SubscriptionUpgradeResult, SubscriptionError> {
        // トランザクション開始
        let txn = self.db.begin().await?;
        
        let result = async {
            // 現在のサブスクリプション取得
            let user = self.user_repo
                .find_by_id(user_id)
                .await?
                .ok_or(SubscriptionError::UserNotFound)?;
            
            let current_tier = user.subscription_tier;
            
            // アップグレード可能性チェック
            if new_tier <= current_tier {
                return Err(SubscriptionError::InvalidUpgrade);
            }
            
            // 支払い処理
            let payment_result = self.billing_service
                .process_payment(user_id, new_tier, payment_method)
                .await?;
            
            // サブスクリプション更新
            self.user_repo
                .update_subscription(&txn, user_id, new_tier)
                .await?;
            
            // 履歴記録
            self.subscription_history_repo
                .create(&txn, SubscriptionHistory {
                    id: Uuid::new_v4(),
                    user_id,
                    previous_tier: current_tier,
                    new_tier,
                    changed_at: Utc::now(),
                    changed_by: user_id,
                    reason: "User upgrade".to_string(),
                    payment_id: Some(payment_result.payment_id),
                })
                .await?;
            
            // 新機能の有効化
            self.enable_tier_features(&txn, user_id, new_tier).await?;
            
            Ok(SubscriptionUpgradeResult {
                new_tier,
                payment_id: payment_result.payment_id,
                activated_features: self.get_new_features(current_tier, new_tier),
            })
        }
        .await;
        
        match result {
            Ok(upgrade_result) => {
                txn.commit().await?;
                
                // 非同期で通知送信
                self.notify_subscription_change(user_id, upgrade_result.clone()).await;
                
                Ok(upgrade_result)
            }
            Err(e) => {
                txn.rollback().await?;
                Err(e)
            }
        }
    }
}
```

### 2. ダウングレード処理（制約チェック付き）

```rust
impl SubscriptionService {
    pub async fn downgrade_subscription(
        &self,
        user_id: Uuid,
        new_tier: SubscriptionTier,
    ) -> Result<SubscriptionDowngradeResult, SubscriptionError> {
        // 現在の使用状況を確認
        let usage = self.get_current_usage(user_id).await?;
        let new_limits = new_tier.features();
        
        // 制約違反チェック
        let violations = self.check_downgrade_violations(&usage, &new_limits);
        if !violations.is_empty() {
            return Err(SubscriptionError::DowngradeViolations(violations));
        }
        
        // トランザクション開始
        let txn = self.db.begin().await?;
        
        let result = async {
            // サブスクリプション更新
            self.user_repo
                .update_subscription(&txn, user_id, new_tier)
                .await?;
            
            // 機能の無効化
            self.disable_tier_features(&txn, user_id, new_tier).await?;
            
            // 履歴記録
            self.subscription_history_repo
                .create(&txn, SubscriptionHistory {
                    id: Uuid::new_v4(),
                    user_id,
                    previous_tier: usage.current_tier,
                    new_tier,
                    changed_at: Utc::now(),
                    changed_by: user_id,
                    reason: "User downgrade".to_string(),
                    payment_id: None,
                })
                .await?;
            
            Ok(SubscriptionDowngradeResult {
                new_tier,
                disabled_features: self.get_disabled_features(usage.current_tier, new_tier),
                next_billing_date: self.calculate_next_billing_date(user_id).await?,
            })
        }
        .await;
        
        match result {
            Ok(downgrade_result) => {
                txn.commit().await?;
                Ok(downgrade_result)
            }
            Err(e) => {
                txn.rollback().await?;
                Err(e)
            }
        }
    }
    
    fn check_downgrade_violations(
        &self,
        usage: &UserUsage,
        new_limits: &SubscriptionFeatures,
    ) -> Vec<DowngradeViolation> {
        let mut violations = Vec::new();
        
        if usage.task_count > new_limits.max_tasks {
            violations.push(DowngradeViolation {
                resource: "tasks".to_string(),
                current: usage.task_count,
                limit: new_limits.max_tasks,
                message: format!(
                    "You have {} tasks but the new plan only allows {}",
                    usage.task_count, new_limits.max_tasks
                ),
            });
        }
        
        if usage.team_count > new_limits.max_teams {
            violations.push(DowngradeViolation {
                resource: "teams".to_string(),
                current: usage.team_count,
                limit: new_limits.max_teams,
                message: format!(
                    "You have {} teams but the new plan only allows {}",
                    usage.team_count, new_limits.max_teams
                ),
            });
        }
        
        violations
    }
}
```

## 組織レベルのサブスクリプション管理

### 1. 組織サブスクリプションモデル

```rust
pub struct OrganizationSubscription {
    pub organization_id: Uuid,
    pub subscription_tier: SubscriptionTier,
    pub seats: u32,  // ライセンス数
    pub used_seats: u32,
    pub billing_cycle: BillingCycle,
    pub next_billing_date: DateTime<Utc>,
}

impl OrganizationSubscriptionService {
    pub async fn add_member_to_organization(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        inviter_id: Uuid,
    ) -> Result<(), SubscriptionError> {
        // 組織のサブスクリプション確認
        let org_subscription = self.get_organization_subscription(org_id).await?;
        
        // シート数チェック
        if org_subscription.used_seats >= org_subscription.seats {
            return Err(SubscriptionError::NoAvailableSeats);
        }
        
        // メンバー追加
        self.org_repo.add_member(org_id, user_id).await?;
        
        // 使用シート数更新
        self.update_used_seats(org_id, org_subscription.used_seats + 1).await?;
        
        // ユーザーのサブスクリプションを組織レベルに更新
        self.sync_user_subscription(user_id, org_subscription.subscription_tier).await?;
        
        Ok(())
    }
}
```

### 2. 請求とライセンス管理

```rust
pub struct BillingService {
    payment_provider: Arc<dyn PaymentProvider>,
    invoice_service: Arc<dyn InvoiceService>,
}

impl BillingService {
    pub async fn process_organization_billing(
        &self,
        org_id: Uuid,
    ) -> Result<BillingResult, BillingError> {
        let org = self.get_organization(org_id).await?;
        let subscription = self.get_organization_subscription(org_id).await?;
        
        // 使用量計算
        let usage = self.calculate_organization_usage(org_id).await?;
        
        // 請求額計算
        let amount = self.calculate_billing_amount(
            subscription.subscription_tier,
            subscription.seats,
            usage,
        );
        
        // 請求書生成
        let invoice = self.invoice_service
            .create_invoice(InvoiceData {
                organization_id: org_id,
                amount,
                billing_period: self.get_current_billing_period(&subscription),
                line_items: self.generate_line_items(&subscription, &usage),
            })
            .await?;
        
        // 支払い処理
        let payment_result = self.payment_provider
            .charge_organization(org_id, amount, invoice.id)
            .await?;
        
        // 次回請求日更新
        self.update_next_billing_date(org_id, subscription.billing_cycle).await?;
        
        Ok(BillingResult {
            invoice_id: invoice.id,
            payment_id: payment_result.payment_id,
            amount,
            next_billing_date: self.calculate_next_billing_date(&subscription),
        })
    }
}
```

## クォータ管理と制限

### 1. リアルタイムクォータチェック

```rust
pub struct QuotaService {
    cache: Arc<dyn Cache>,
    quota_repo: Arc<dyn QuotaRepository>,
}

impl QuotaService {
    pub async fn check_and_increment_quota(
        &self,
        user_id: Uuid,
        resource_type: ResourceType,
        amount: u32,
    ) -> Result<(), QuotaError> {
        // キャッシュからクォータ情報取得
        let quota_key = format!("quota:{}:{}", user_id, resource_type);
        let current_usage: u32 = self.cache
            .get(&quota_key)
            .await?
            .unwrap_or(0);
        
        // ユーザーのサブスクリプション取得
        let user = self.get_user(user_id).await?;
        let limits = user.subscription_tier.features();
        
        let max_allowed = match resource_type {
            ResourceType::Task => limits.max_tasks,
            ResourceType::Team => limits.max_teams,
            ResourceType::ApiCall => limits.rate_limit_per_minute,
        };
        
        // クォータチェック
        if current_usage + amount > max_allowed {
            return Err(QuotaError::QuotaExceeded {
                resource: resource_type,
                current: current_usage,
                requested: amount,
                limit: max_allowed,
            });
        }
        
        // 使用量更新（アトミック操作）
        let new_usage = self.cache
            .increment(&quota_key, amount)
            .await?;
        
        // 永続化（非同期）
        tokio::spawn(async move {
            if let Err(e) = self.quota_repo.update_usage(user_id, resource_type, new_usage).await {
                tracing::error!("Failed to persist quota usage: {}", e);
            }
        });
        
        Ok(())
    }
}
```

### 2. レート制限の実装

```rust
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;

pub struct RateLimitService {
    limiters: DashMap<(Uuid, String), Arc<RateLimiter<String>>>,
}

impl RateLimitService {
    pub fn get_or_create_limiter(
        &self,
        user_id: Uuid,
        endpoint: &str,
        subscription_tier: SubscriptionTier,
    ) -> Arc<RateLimiter<String>> {
        let key = (user_id, endpoint.to_string());
        
        self.limiters
            .entry(key)
            .or_insert_with(|| {
                let quota = match subscription_tier {
                    SubscriptionTier::Free => Quota::per_minute(NonZeroU32::new(10).unwrap()),
                    SubscriptionTier::Pro => Quota::per_minute(NonZeroU32::new(100).unwrap()),
                    SubscriptionTier::Enterprise => Quota::per_minute(NonZeroU32::new(1000).unwrap()),
                };
                
                Arc::new(RateLimiter::direct(quota))
            })
            .clone()
    }
    
    pub async fn check_rate_limit(
        &self,
        user_id: Uuid,
        endpoint: &str,
        subscription_tier: SubscriptionTier,
    ) -> Result<(), RateLimitError> {
        let limiter = self.get_or_create_limiter(user_id, endpoint, subscription_tier);
        let key = format!("{}:{}", user_id, endpoint);
        
        match limiter.check_key(&key) {
            Ok(_) => Ok(()),
            Err(_) => {
                let retry_after = limiter
                    .rate_limiter()
                    .until_key_ready(&key)
                    .await;
                
                Err(RateLimitError::RateLimitExceeded { retry_after })
            }
        }
    }
}
```

## 通知とコミュニケーション

### 1. サブスクリプション関連通知

```rust
pub enum SubscriptionNotification {
    UpgradeSuccess { new_tier: SubscriptionTier },
    DowngradeScheduled { new_tier: SubscriptionTier, effective_date: DateTime<Utc> },
    QuotaWarning { resource: ResourceType, usage_percentage: f32 },
    PaymentFailed { retry_date: DateTime<Utc> },
    TrialEnding { days_remaining: u32 },
}

impl NotificationService {
    pub async fn send_subscription_notification(
        &self,
        user_id: Uuid,
        notification: SubscriptionNotification,
    ) -> Result<(), NotificationError> {
        let user = self.get_user(user_id).await?;
        
        let (subject, body, priority) = match notification {
            SubscriptionNotification::UpgradeSuccess { new_tier } => (
                format!("Welcome to {} plan!", new_tier),
                self.render_upgrade_email(&user, new_tier),
                NotificationPriority::High,
            ),
            SubscriptionNotification::QuotaWarning { resource, usage_percentage } => (
                format!("You've used {}% of your {} quota", usage_percentage, resource),
                self.render_quota_warning_email(&user, resource, usage_percentage),
                NotificationPriority::Medium,
            ),
            // ... 他の通知タイプ
        };
        
        // メール送信
        self.email_service
            .send_email(EmailMessage {
                to: user.email,
                subject,
                body,
                priority,
            })
            .await?;
        
        // アプリ内通知
        self.create_in_app_notification(user_id, notification).await?;
        
        Ok(())
    }
}
```

## まとめ

効果的なサブスクリプション管理は、SaaSビジネスの成功の鍵です。階層的な機能制限、スムーズなアップグレード/ダウングレード処理、正確なクォータ管理、適切な通知により、ユーザー体験を向上させながら、ビジネスの成長を支援できます。