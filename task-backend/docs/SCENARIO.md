# ユーザー登録後のシナリオ完全ガイド

このドキュメントでは、ユーザーがアカウント作成後に体験する全てのシナリオを網羅的に説明します。

## 概要

このRust製タスク管理APIは、**動的パーミッションシステム**を採用しており、ユーザーの役割とサブスクリプション階層によって同一エンドポイントが異なる応答を返すことが特徴です。

## シナリオ1: 新規ユーザー登録直後

### 1.1 基本的な登録フロー

```mermaid
sequenceDiagram
    participant User as ユーザー
    participant API as API Server
    participant DB as Database
    participant Auth as Auth Service

    User->>API: POST /auth/signup
    Note over User,API: {email, username, password}
    
    API->>Auth: パスワード強度チェック
    Auth-->>API: 強度OK
    
    API->>DB: ユーザー重複チェック
    DB-->>API: 重複なし
    
    API->>Auth: Argon2ハッシュ化
    Auth-->>API: ハッシュ化完了
    
    API->>DB: ユーザー作成（トランザクション）
    Note over API,DB: デフォルト値設定
    DB-->>API: ユーザー作成成功
    
    API->>Auth: JWT生成
    Auth-->>API: アクセス・リフレッシュトークン
    
    API-->>User: 登録完了レスポンス
    Note over User,API: ユーザー情報 + トークン
```

### 1.2 登録直後のデフォルト設定

新規ユーザーには以下のデフォルト値が設定されます：

| 項目 | デフォルト値 | 説明 |
|------|-------------|------|
| `subscription_tier` | `"free"` | 無料プラン |
| `role` | `"member"` | 一般ユーザー権限 |
| `is_active` | `true` | 即座にログイン可能 |
| `email_verified` | `false` | メール認証は未完了 |
| `permission_scope` | `Own` | 自分のデータのみアクセス可能 |
| `task_quota` | `100` | 最大100タスク |
| `rate_limit` | `10/分` | 分間10リクエスト |

### 1.3 即座に利用可能な機能

```mermaid
graph TD
    A[新規登録完了] --> B[即座にログイン可能]
    B --> C[基本的なタスク管理]
    C --> D[タスク作成・編集・削除]
    C --> E[自分のタスク一覧表示]
    C --> F[基本的なフィルタリング]
    
    B --> G[アカウント情報表示]
    B --> H[プロフィール編集]
    
    A --> I[制限事項]
    I --> J[最大100タスク]
    I --> K[基本機能のみ]
    I --> L[自分のデータのみ]
```

## シナリオ2: サブスクリプション階層による機能差分

### 2.1 Free Tier（無料プラン）

```mermaid
graph LR
    A[Free Tier] --> B[制限事項]
    B --> C[最大100タスク]
    B --> D[10リクエスト/分]
    B --> E[自分のデータのみ]
    B --> F[基本機能のみ]
    
    A --> G[利用可能機能]
    G --> H[タスクCRUD]
    G --> I[基本フィルタ]
    G --> J[プロフィール管理]
```

**利用可能なエンドポイント例：**
- `GET /tasks` - 最大100件の自分のタスク
- `POST /tasks` - タスク作成
- `GET /tasks/{id}` - 自分のタスク詳細
- `PUT /tasks/{id}` - 自分のタスク更新
- `DELETE /tasks/{id}` - 自分のタスク削除

### 2.2 Pro Tier（プロプラン）

```mermaid
graph LR
    A[Pro Tier] --> B[拡張機能]
    B --> C[最大10,000タスク]
    B --> D[100リクエスト/分]
    B --> E[チーム範囲アクセス]
    B --> F[高度なフィルタリング]
    B --> G[データエクスポート]
    B --> H[バッチ操作]
    
    A --> I[チーム機能]
    I --> J[最大10メンバー]
    I --> K[チームタスク管理]
    I --> L[メンバー招待]
```

**Pro限定機能：**
- 高度な検索とフィルタリング
- CSV/JSONエクスポート
- チームタスクの閲覧・管理
- バッチ操作（最大1,000件）

### 2.3 Enterprise Tier（エンタープライズプラン）

```mermaid
graph LR
    A[Enterprise Tier] --> B[無制限機能]
    B --> C[無制限タスク]
    B --> D[レート制限なし]
    B --> E[全体範囲アクセス]
    B --> F[全機能利用可能]
    
    A --> G[管理機能]
    G --> H[最大100メンバー]
    G --> I[組織管理]
    G --> J[高度な分析]
    G --> K[SSO統合]
    G --> L[優先サポート]
```

**Enterprise限定機能：**
- 無制限のタスクとリクエスト
- 組織全体の管理機能
- 高度な分析とレポート
- カスタムロールとパーミッション

## シナリオ3: 動的パーミッションシステム

### 3.1 同一エンドポイントの異なる挙動

```mermaid
graph TD
    A[GET /tasks/dynamic] --> B{ユーザーの権限チェック}
    
    B -->|Free + Own| C[Limited Response]
    C --> C1[最大100件]
    C --> C2[基本情報のみ]
    C --> C3[クォータ情報付き]
    
    B -->|Pro + Team| D[Enhanced Response]
    D --> D1[最大10,000件]
    D --> D2[高度な機能情報]
    D --> D3[エクスポート可能]
    
    B -->|Enterprise + Global| E[Unlimited Response]
    E --> E1[無制限]
    E --> E2[全機能利用可能]
    E --> E3[バルク操作可能]
    
    B -->|Admin| F[Admin Response]
    F --> F1[全ユーザーのデータ]
    F --> F2[管理機能付き]
    F --> F3[統計情報付き]
```

## シナリオ4: チーム・組織管理

### 4.1 チーム作成・参加フロー

```mermaid
sequenceDiagram
    participant Owner as チームオーナー
    participant Member as メンバー
    participant API as API Server
    participant Email as メールサービス
    
    Owner->>API: POST /teams（チーム作成）
    API-->>Owner: チーム作成完了
    
    Owner->>API: POST /teams/{id}/members（メンバー招待）
    API->>Email: 招待メール送信
    Email-->>Member: 招待通知
    
    Member->>API: POST /teams/join（招待承認）
    API-->>Member: チーム参加完了
    API-->>Owner: 参加通知
    
    Note over Owner,Member: チーム内でのタスク共有開始
```

### 4.2 チーム内での権限管理

```mermaid
graph TD
    A[チーム] --> B[Owner（オーナー）]
    A --> C[Admin（管理者）]
    A --> D[Member（メンバー）]
    A --> E[Viewer（閲覧者）]
    
    B --> B1[全権限]
    B --> B2[メンバー管理]
    B --> B3[チーム設定変更]
    
    C --> C1[メンバー管理]
    C --> C2[タスク管理]
    C --> C3[設定変更]
    
    D --> D1[タスク作成・編集]
    D --> D2[チームタスク表示]
    
    E --> E1[タスク表示のみ]
```

## シナリオ5: サブスクリプション変更フロー

### 5.1 アップグレードシナリオ

```mermaid
stateDiagram-v2
    [*] --> Free
    Free --> Pro : 自己アップグレード
    Pro --> Enterprise : 自己アップグレード
    Free --> Enterprise : 直接アップグレード
    
    Free --> Free_Admin : 管理者による変更
    Pro --> Pro_Admin : 管理者による変更
    Enterprise --> Enterprise_Admin : 管理者による変更
    
    state Free {
        [*] --> BasicFeatures
        BasicFeatures --> QuotaReached
        QuotaReached --> UpgradePrompt
    }
    
    state Pro {
        [*] --> EnhancedFeatures
        EnhancedFeatures --> TeamManagement
        TeamManagement --> AdvancedAnalytics
    }
    
    state Enterprise {
        [*] --> UnlimitedFeatures
        UnlimitedFeatures --> OrganizationManagement
        OrganizationManagement --> CustomIntegration
    }
```

### 5.2 ダウングレード時の制限処理

```mermaid
sequenceDiagram
    participant User as ユーザー
    participant API as API Server
    participant DB as Database
    participant Cleanup as クリーンアップサービス
    
    User->>API: サブスクリプションダウングレード
    API->>DB: 現在のデータ量チェック
    
    alt データ量が新しい制限を超過
        DB-->>API: 制限超過エラー
        API-->>User: ダウングレード不可（データ削除が必要）
    else データ量OK
        API->>DB: サブスクリプション更新
        API->>Cleanup: 機能制限の適用
        Cleanup->>DB: 機能無効化
        DB-->>API: 更新完了
        API-->>User: ダウングレード完了
    end
```

## シナリオ6: 認証・セキュリティフロー

### 6.1 トークン管理

```mermaid
graph TD
    A[ログイン] --> B[アクセストークン取得]
    B --> C[15分間有効]
    C --> D{トークン期限切れ？}
    
    D -->|Yes| E[リフレッシュトークン使用]
    E --> F[新しいアクセストークン取得]
    F --> G[7日間有効]
    
    D -->|No| H[API継続使用]
    
    G --> I{リフレッシュトークン期限切れ？}
    I -->|Yes| J[再ログイン必要]
    I -->|No| K[自動更新]
    
    J --> A
    K --> H
```

### 6.2 パスワードリセットフロー

```mermaid
sequenceDiagram
    participant User as ユーザー
    participant API as API Server
    participant Email as メールサービス
    participant DB as Database
    
    User->>API: POST /auth/forgot-password
    API->>DB: ユーザー存在確認
    DB-->>API: ユーザー確認
    
    API->>DB: リセットトークン生成・保存
    API->>Email: リセットメール送信
    Email-->>User: リセットリンク
    
    User->>API: POST /auth/reset-password
    Note over User,API: トークン + 新パスワード
    
    API->>DB: トークン検証
    DB-->>API: 有効なトークン
    
    API->>DB: パスワード更新
    API->>DB: リセットトークン無効化
    DB-->>API: 更新完了
    
    API-->>User: パスワード変更完了
```

## シナリオ7: エラーハンドリング

### 7.1 権限不足エラー

```mermaid
graph TD
    A[APIリクエスト] --> B{権限チェック}
    
    B -->|権限あり| C[正常処理]
    B -->|権限なし| D[403 Forbidden]
    
    D --> E{具体的なエラー}
    E -->|クォータ超過| F[クォータ制限エラー]
    E -->|機能制限| G[機能制限エラー]
    E -->|スコープ外| H[アクセス範囲エラー]
    
    F --> I[アップグレード促進]
    G --> I
    H --> J[適切なスコープ説明]
```

### 7.2 サブスクリプション制限エラー

```mermaid
graph TD
    A[機能利用試行] --> B{サブスクリプション確認}
    
    B -->|Free| C{Free制限チェック}
    B -->|Pro| D{Pro制限チェック}
    B -->|Enterprise| E[制限なし]
    
    C -->|制限内| F[機能利用OK]
    C -->|制限超過| G[Free制限エラー]
    
    D -->|制限内| F
    D -->|制限超過| H[Pro制限エラー]
    
    G --> I[Proプランへのアップグレード提案]
    H --> J[Enterpriseプランへのアップグレード提案]
```

## シナリオ8: 管理者権限による特別フロー

### 8.1 管理者による強制変更

```mermaid
sequenceDiagram
    participant Admin as 管理者
    participant API as API Server
    participant DB as Database
    participant User as 対象ユーザー
    
    Admin->>API: 管理者権限でユーザー変更
    API->>DB: 管理者権限確認
    DB-->>API: 権限確認OK
    
    API->>DB: ユーザー情報強制変更
    Note over API,DB: サブスクリプション・役割・状態
    
    DB-->>API: 変更完了
    API->>User: 変更通知（メール）
    API-->>Admin: 変更完了レスポンス
    
    Note over Admin,User: 監査ログ記録
```

## まとめ

このシステムは、**ユーザーの成長に合わせて段階的に機能を開放する**設計となっています：

1. **新規登録時**：即座に基本機能が利用可能
2. **Free Tier**：個人利用に十分な基本機能
3. **Pro Tier**：チーム協働と高度な機能
4. **Enterprise Tier**：組織全体の管理と無制限利用

各段階で同一APIが異なる応答を返すことで、ユーザーは段階的な機能拡張を体験でき、開発者はエンドポイントの一貫性を保てます。

この動的パーミッションシステムにより、**スケーラブルで保守性の高いサブスクリプションベースSaaS**を実現しています。