# 🎯 実践デモ：新しいAPIエンドポイント追加

## シナリオ：「タスク検索API」を追加する

実際の開発フローを体験してみましょう。

### 🚀 **STEP 1: 環境準備**

```bash
# プロジェクトディレクトリに移動
cd rust-backend-api-first-lesson

# 開発環境セットアップ
make dev-setup

# データベース起動 & マイグレーション
make dev
```

期待する結果：
```
✅ PostgreSQL起動完了
✅ マイグレーション適用完了  
✅ サーバー起動完了（localhost:3000）
```

---

### 🚀 **STEP 2: 現在のAPIを確認**

```bash
# 別ターミナルで動作確認
curl http://localhost:3000/health
# 結果: OK

# 既存のタスクAPI確認
curl http://localhost:3000/tasks | jq
# 結果: [] (空配列)

# テストデータ作成
curl -X POST http://localhost:3000/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "title": "重要なタスク",
    "description": "これは重要なタスクです",
    "status": "todo"
  }' | jq

curl -X POST http://localhost:3000/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "title": "普通のタスク", 
    "description": "これは普通のタスクです",
    "status": "in_progress"
  }' | jq
```

---

### 🚀 **STEP 3: 新機能のテスト追加（TDD的アプローチ）**

新しい検索APIのテストを先に書いてみます：

```rust
// task-backend/tests/integration/api_tests.rs に追加

#[tokio::test]
async fn test_search_tasks_by_keyword() {
    let (app, _schema_name, _db) = setup_test_app().await;

    // テストデータ作成
    create_test_task_with_data(&app, "重要なプロジェクト", "プロジェクト管理").await;
    create_test_task_with_data(&app, "普通のタスク", "日常業務").await;
    create_test_task_with_data(&app, "重要な会議", "プロジェクト関連").await;

    // 検索テスト
    let req = Request::builder()
        .uri("/tasks/search?q=重要")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let tasks: Vec<TaskDto> = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(tasks.len(), 2); // "重要"を含むタスクが2つ
}
```

---

### 🚀 **STEP 4: テスト実行（まだ失敗するはず）**

```bash
# テスト実行
cargo test test_search_tasks_by_keyword

# 結果: 失敗（エンドポイントが存在しないため）
```

---

### 🚀 **STEP 5: APIエンドポイント実装**

#### 5.1 DTOに検索パラメータ追加
```rust
// task-backend/src/api/dto/task_dto.rs に追加

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct TaskSearchDto {
    pub q: Option<String>,           // 検索キーワード
    pub status: Option<String>,      // ステータスフィルター
    pub limit: Option<u64>,          // 件数制限
}
```

#### 5.2 リポジトリに検索メソッド追加
```rust
// task-backend/src/repository/task_repository.rs に追加

impl TaskRepository {
    pub async fn search_tasks(
        &self, 
        search_params: &TaskSearchDto
    ) -> Result<Vec<task_model::Model>, DbErr> {
        self.prepare_connection().await?;
        
        let mut query = TaskEntity::find();
        let mut conditions = Condition::all();

        // キーワード検索
        if let Some(keyword) = &search_params.q {
            let keyword_condition = Condition::any()
                .add(task_model::Column::Title.contains(keyword))
                .add(task_model::Column::Description.contains(keyword));
            conditions = conditions.add(keyword_condition);
        }

        // ステータスフィルター
        if let Some(status) = &search_params.status {
            conditions = conditions.add(task_model::Column::Status.eq(status.clone()));
        }

        query = query.filter(conditions);
        
        // 件数制限
        if let Some(limit) = search_params.limit {
            query = query.limit(limit);
        }

        query.all(&self.db).await
    }
}
```

#### 5.3 サービスに検索メソッド追加
```rust
// task-backend/src/service/task_service.rs に追加

impl TaskService {
    pub async fn search_tasks(&self, search_params: TaskSearchDto) -> AppResult<Vec<TaskDto>> {
        let tasks = self.repo.search_tasks(&search_params).await?;
        Ok(tasks.into_iter().map(Into::into).collect())
    }
}
```

#### 5.4 ハンドラーに検索エンドポイント追加
```rust
// task-backend/src/api/handlers/task_handler.rs に追加

pub async fn search_tasks_handler(
    State(app_state): State<AppState>,
    Query(search_params): Query<TaskSearchDto>,
) -> AppResult<Json<Vec<TaskDto>>> {
    let tasks = app_state.task_service.search_tasks(search_params).await?;
    Ok(Json(tasks))
}

// ルーターに追加
pub fn task_router(app_state: AppState) -> Router {
    Router::new()
        .route("/tasks", get(list_tasks_handler).post(create_task_handler))
        .route("/tasks/search", get(search_tasks_handler))  // 新規追加
        .route("/tasks/paginated", get(list_tasks_paginated_handler))
        // ... 他のルート
        .with_state(app_state)
}
```

---

### 🚀 **STEP 6: 実装後のテスト**

```bash
# フォーマット & 静的解析
make fmt && make clippy

# テスト実行
cargo test test_search_tasks_by_keyword
# 結果: 成功するはず

# 全テスト実行
make test
```

---

### 🚀 **STEP 7: 手動での動作確認**

```bash
# サーバー再起動
make run

# 検索API動作確認
curl "http://localhost:3000/tasks/search?q=重要" | jq
# 結果: 重要を含むタスク一覧

curl "http://localhost:3000/tasks/search?status=todo" | jq  
# 結果: todoステータスのタスク一覧

curl "http://localhost:3000/tasks/search?q=プロジェクト&status=todo&limit=5" | jq
# 結果: 複合検索結果
```

---

### 🚀 **STEP 8: コンテナでの動作確認**

```bash
# Dockerイメージビルド
make docker-build

# Docker Compose環境で起動
make docker-run

# コンテナでのAPI確認
curl "http://localhost:3000/tasks/search?q=重要" | jq
```

---

### 🚀 **STEP 9: エラーハンドリング & エッジケース**

#### 9.1 バリデーション追加
```rust
// ハンドラーでのバリデーション
pub async fn search_tasks_handler(
    State(app_state): State<AppState>,
    Query(search_params): Query<TaskSearchDto>,
) -> AppResult<Json<Vec<TaskDto>>> {
    // 検索キーワードが短すぎる場合のバリデーション
    if let Some(ref keyword) = search_params.q {
        if keyword.trim().len() < 2 {
            return Err(AppError::ValidationError(
                "Search keyword must be at least 2 characters".to_string()
            ));
        }
    }
    
    let tasks = app_state.task_service.search_tasks(search_params).await?;
    Ok(Json(tasks))
}
```

#### 9.2 エッジケーステスト
```bash
# 短すぎるキーワード
curl "http://localhost:3000/tasks/search?q=a" | jq
# 結果: バリデーションエラー

# 存在しないステータス
curl "http://localhost:3000/tasks/search?status=nonexistent" | jq
# 結果: 空配列
```

---

### 🚀 **STEP 10: 完了チェック**

```bash
# 最終的な全体テスト
make ci-check

# パフォーマンステスト
make profile

# ドキュメント生成
make docs
```

---

## 🎯 **このデモで学べること**

1. **TDD的アプローチ**: テストを先に書いて、実装を後から行う
2. **段階的実装**: DTO → Repository → Service → Handler → Router の順
3. **継続的な動作確認**: 各段階での手動テスト
4. **エラーハンドリング**: バリデーションとエッジケースの考慮
5. **コンテナ化**: 開発環境と本番環境の一致確認

## 💡 **次のステップ**

このワークフローに慣れたら：
- 認証機能の追加
- ページネーション対応
- ソート機能の拡張
- キャッシュ機能の追加
- メトリクス・ログの強化

実際にこの流れで開発してみてください！