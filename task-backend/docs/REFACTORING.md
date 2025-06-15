## ✅ リファクタリング項目構造化一覧

### 🔷 A. 日時処理の不統一

- **課題**：UTC 時刻処理が個別実装されており、整合性が取れていない
- **改善案**：

  - `chrono::Utc` や `time::OffsetDateTime` の統一使用
  - `serde_with::TimestampSecondsWithFrac` などによるシリアライズ統一
  - 共通の日時ユーティリティを導入（例：`time_utils::now_utc()`）

---

### 🔷 B. エラーハンドリングの改善

#### 🔸 現在の実装

```rust
pub enum AppError { ... } // カスタム定義
```

#### ✅ Rust ベストプラクティス

- `thiserror` + `anyhow` を活用しエラー定義を簡潔・拡張可能に：

```rust
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Validation failed: {0}")]
    Validation(#[from] validator::ValidationErrors),

    #[error("Database error")]
    Database(#[from] sea_orm::DbErr),
}
```

---

### 🔷 C. 型安全性の向上

#### 🔸 現在の実装

```rust
pub status: String // 状態を文字列で管理
```

#### ✅ Rust ベストプラクティス

- `enum` による状態管理で **誤値入力をコンパイル時に防止**：

```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Completed,
    Cancelled,
}
```

---

### 🔷 D. 型による権限管理（ゼロコスト抽象化）

#### 🔸 現在の実装

```rust
if !user.is_admin() {
    return Err(...);
}
```

#### ✅ Rust ベストプラクティス

- **型レベルでのアクセス制御**により、権限違反を事前に排除：

```rust
pub struct Admin<T>(T);
pub struct Member<T>(T);

impl<T> Admin<T> {
    pub fn access_all_tasks(&self) -> &T { ... }
}
```

---

### 🔷 E. 並行処理の導入

#### 🔸 現在の実装（逐次処理）

```rust
let tasks = self.repo.find_all().await?;
let users = self.user_repo.find_all().await?;
```

#### ✅ Rust ベストプラクティス

- `tokio::join!` による非同期並列化で高速化：

```rust
let (tasks, users) = tokio::join!(
    self.repo.find_all(),
    self.user_repo.find_all()
);
```

---

### 🔷 F. メモリ効率の改善（遅延評価）

#### 🔸 現在の実装

```rust
pub async fn list_tasks(&self) -> AppResult<Vec<TaskDto>>
```

#### ✅ Rust ベストプラクティス

- **Stream や Iterator** によるメモリ効率の良い逐次処理：

```rust
pub fn list_tasks(&self) -> impl Stream<Item = Result<TaskDto, AppError>>
```

---

## ✅ 統合カテゴリマップ

| 分類               | 改善項目              | 技術的観点       |
| ------------------ | --------------------- | ---------------- |
| 日時・共通処理     | UTC 統一              | 可読性・一貫性   |
| エラーハンドリング | thiserror + anyhow    | 保守性・拡張性   |
| 型安全             | enum による状態管理   | コンパイル時安全 |
| アクセス制御       | 型レベルでの権限管理  | 抽象化・安全性   |
| 並行性             | tokio::join           | パフォーマンス   |
| メモリ効率         | Stream による遅延処理 | スケーラビリティ |
