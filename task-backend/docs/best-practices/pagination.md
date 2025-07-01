# ページネーションベストプラクティス

## 概要

このドキュメントでは、Rust製バックエンドAPIにおけるページネーション実装のベストプラクティスをまとめています。

## 実装パターン

### 1. データベースレベルでのページネーション（推奨）

```rust
// リポジトリ層
pub async fn find_all_paginated(
    &self,
    page: i32,
    per_page: i32,
) -> Result<Vec<Model>, DbErr> {
    let page_size = std::cmp::min(per_page as u64, 100); // 最大件数制限
    let offset = ((page - 1) * per_page) as u64;

    Entity::find()
        .order_by(Column::CreatedAt, Order::Desc)
        .limit(page_size)
        .offset(offset)
        .all(&self.db)
        .await
}

// 総件数取得（別クエリ）
pub async fn count_all(&self) -> Result<u64, DbErr> {
    Entity::find().count(&self.db).await
}
```

### 2. サービス層での実装

```rust
// サービス層
pub async fn list_items_paginated(
    &self,
    page: i32,
    per_page: i32,
) -> AppResult<(Vec<Item>, usize)> {
    let items = self.repo.find_all_paginated(page, per_page).await?;
    let total_count = self.repo.count_all().await? as usize;
    Ok((items, total_count))
}
```

## アンチパターン

### ❌ メモリ内ページネーション（避けるべき）

```rust
// 悪い例：全データをメモリに読み込んでからページネーション
let all_items = self.repo.find_all().await?;
let total_count = all_items.len();
let offset = ((page - 1) * per_page) as usize;
let paginated = all_items.into_iter().skip(offset).take(per_page).collect();
```

**問題点**：
- メモリ消費が大きい
- パフォーマンスが悪い
- スケーラビリティがない

## 実装ガイドライン

### 1. 最大件数制限

```rust
const MAX_PAGE_SIZE: u64 = 100;
let page_size = std::cmp::min(per_page as u64, MAX_PAGE_SIZE);
```

### 2. カーソルベースページネーション（大規模データセット用）

```rust
pub struct CursorPagination {
    pub cursor: Option<String>, // Base64エンコードされたカーソル
    pub limit: u32,
}

// 実装例
pub async fn find_with_cursor(
    &self,
    cursor: Option<String>,
    limit: u32,
) -> Result<(Vec<Model>, Option<String>), DbErr> {
    let mut query = Entity::find();
    
    if let Some(cursor) = cursor {
        let decoded_id = decode_cursor(&cursor)?;
        query = query.filter(Column::Id.gt(decoded_id));
    }
    
    let items = query
        .order_by(Column::Id, Order::Asc)
        .limit(limit as u64 + 1) // 次のページがあるか確認するため+1
        .all(&self.db)
        .await?;
    
    let has_next = items.len() > limit as usize;
    let items = if has_next {
        items[..limit as usize].to_vec()
    } else {
        items
    };
    
    let next_cursor = if has_next {
        Some(encode_cursor(&items.last().unwrap().id))
    } else {
        None
    };
    
    Ok((items, next_cursor))
}
```

### 3. レスポンス構造

```rust
#[derive(Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

#[derive(Serialize)]
pub struct PaginationMeta {
    pub current_page: u32,
    pub per_page: u32,
    pub total_pages: u32,
    pub total_count: u64,
    pub has_next: bool,
    pub has_prev: bool,
}
```

## パフォーマンス最適化

### 1. インデックスの活用

```sql
-- ページネーションで使用するカラムにインデックスを作成
CREATE INDEX idx_created_at ON users(created_at DESC);
CREATE INDEX idx_id ON users(id);
```

### 2. カウントクエリの最適化

```rust
// 条件付きカウントの場合、同じ条件を適用
pub async fn count_by_filter(
    &self,
    filter: &FilterOptions,
) -> Result<u64, DbErr> {
    let mut query = Entity::find();
    apply_filters(&mut query, filter);
    query.count(&self.db).await
}
```

### 3. 結合クエリでのページネーション

```rust
// 結合を含むクエリでもLIMIT/OFFSETを適用
let results = UserEntity::find()
    .join(JoinType::InnerJoin, user_model::Relation::Role.def())
    .select_also(RoleEntity)
    .order_by(user_model::Column::CreatedAt, Order::Desc)
    .limit(page_size)
    .offset(offset)
    .all(&self.db)
    .await?;
```

## 移行ガイド

### 既存のメモリ内ページネーションからの移行

1. **リポジトリ層にページネーションメソッドを追加**
   ```rust
   pub async fn find_all_paginated(...) { }
   pub async fn count_all() { }
   ```

2. **サービス層を更新**
   - 全件取得メソッドの呼び出しを削除
   - ページネーション対応メソッドを使用

3. **APIハンドラーを更新**
   - ページネーションパラメータの検証
   - レスポンス形式の統一

4. **テストを更新**
   - ページネーションロジックのテスト追加
   - エッジケースのテスト（最初/最後のページ、空の結果等）

## まとめ

効率的なページネーションは、スケーラブルなAPIの重要な要素です。データベースレベルでのページネーションを実装し、適切な制限とインデックスを設定することで、パフォーマンスとユーザー体験を向上させることができます。