# 🚀 Rust API を Render + Neon で無料デプロイする完全手順（2025年最新版）

## 📋 事前準備

必要なアカウント：
- [Render](https://render.com) アカウント
- [Neon](https://neon.tech) アカウント
- コンテナイメージ（既にプッシュ済み）

---

## 🗄️ STEP 1: Neon でデータベースセットアップ

### 1.1 Neonにログイン
1. [Neon](https://neon.tech) にアクセス
2. 「Sign Up」または「Sign In」をクリック
3. お好みの認証方法でログイン（GitHub/Google/Email）

### 1.2 新しいプロジェクト作成
1. ダッシュボードで「Create a project」をクリック
2. 以下の設定を入力：
   - **Project name**: `task-backend-db`
   - **Database name**: `neondb`（デフォルト）
   - **Region**: `AWS US East (N. Virginia)` または最寄りのリージョン
3. 「Create project」をクリック

### 1.3 接続情報を取得
1. プロジェクト作成後、「Connection details」が表示される
2. **重要**: **Connection string**をコピーして保存：
   ```
   postgresql://neondb_owner:npg_xxxxx@ep-xxxxx.eastus2.azure.neon.tech/neondb?sslmode=require
   ```

---

## 🌐 STEP 2: Render でWebサービスセットアップ

### 2.1 Renderにログイン
1. [Render](https://render.com) にアクセス
2. 「Get Started」をクリック
3. お好みの認証方法でログイン（GitHub/Google/Email）

### 2.2 新しいWebサービス作成
1. ダッシュボードで「New +」をクリック
2. 「Web Service」を選択
3. **「Deploy an existing image from a registry」**を選択

### 2.3 サービス設定
以下の情報を入力：

| 項目 | 値 |
|------|-----|
| **Image URL** | `ghcr.io/higashi-kota/rust-backend-api-first-lesson:latest` |
| **Name** | `rust-backend-api-first-lesson` |
| **Region** | `Oregon (US West)` または最寄りのリージョン |

### 2.4 Instance Type選択
- **For hobby projects** セクションで
- 「**Free**」を選択（512 MB RAM, 0.1 CPU, $0/month）

### 2.5 Environment Variables設定
「**Environment Variables**」セクションで以下を追加：

| NAME_OF_VARIABLE | value |
|------------------|-------|
| `DATABASE_URL` | Step 1.3で取得したNeonのConnection string |
| `SERVER_ADDR` | `0.0.0.0:10000` |
| `RUST_LOG` | `info` |
| `RUST_BACKTRACE` | `1` |

**重要**: `SERVER_ADDR`は必ず`0.0.0.0:10000`に設定（Renderはポート10000を使用）

### 2.6 その他の設定
| 項目 | 値 | 備考 |
|------|-----|------|
| **Health Check Path** | `/health` | デフォルトのまま |
| **Registry Credential** | `No credential` | パブリックイメージなので不要 |
| **Auto-Deploy** | `On Commit` | お好みで設定 |

### 2.7 サービス作成
1. すべての設定を確認
2. 「**Deploy Web Service**」をクリック
3. デプロイが開始される（5-10分程度）

---

## 🔧 STEP 3: ローカルでマイグレーション実行（重要！）

**⚠️ Renderの無料プランではShellやPre-Deploy Commandが使えないため、ローカルからマイグレーションを実行する必要があります。**

### 3.1 ローカル環境準備
```bash
# プロジェクトディレクトリに移動
cd rust-backend-api-first-lesson

# NeonのDATABASE_URLを環境変数に設定（Step 1.3で取得したものを使用）
export DATABASE_URL="postgresql://neondb_owner:npg_xxxxx@ep-xxxxx.eastus2.azure.neon.tech/neondb?sslmode=require"
```

### 3.2 マイグレーション実行
```bash
# マイグレーション実行
cargo run --package migration -- up
```

**期待する結果**:
```
Applying all pending migrations
Applying migration 'm20250511_073638_create_task_table'
Migration 'm20250511_073638_create_task_table' has been applied
Applying migration 'm20250512_000001_add_task_indexes'
Migration 'm20250512_000001_add_task_indexes' has been applied
```

### 3.3 マイグレーション確認
```bash
# マイグレーション状態確認
cargo run --package migration -- status
```

---

## ✅ STEP 4: 動作確認

### 4.1 Renderサービス確認
1. Renderダッシュボードでサービスが「Live」ステータスになることを確認
2. サービスURLをコピー（例: `https://rust-backend-api-first-lesson-latest.onrender.com`）

### 4.2 ヘルスチェック
```bash
curl https://your-service-url.onrender.com/health
```
**期待する結果**: `OK`

### 4.3 完全なAPI動作確認
```bash
# 1. ヘルスチェック
curl https://rust-backend-api-first-lesson-latest.onrender.com/health

# 2. タスク一覧取得（空の配列）
curl https://rust-backend-api-first-lesson-latest.onrender.com/tasks

# 3. タスク作成
curl -X POST https://rust-backend-api-first-lesson-latest.onrender.com/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "title": "デプロイ成功！",
    "description": "Render + Neon 無料デプロイ完了",
    "status": "todo"
  }' | jq

# 4. 作成したタスクを確認
curl https://rust-backend-api-first-lesson-latest.onrender.com/tasks | jq

# 5. フィルタリング機能テスト
curl "https://rust-backend-api-first-lesson-latest.onrender.com/tasks/filter?status=todo" | jq

# 6. ページネーション機能テスト
curl "https://rust-backend-api-first-lesson-latest.onrender.com/tasks/paginated?page=1&page_size=5" | jq
```

**期待するタスク作成レスポンス**:
```json
{
  "id": "939f3162-f9c0-4f13-9bc3-afd9771c607a",
  "title": "デプロイ成功！",
  "description": "Render + Neon 無料デプロイ完了",
  "status": "todo",
  "due_date": null,
  "created_at": "2025-05-28T13:51:44.784510Z",
  "updated_at": "2025-05-28T13:51:44.784510Z"
}
```

---

## 🔍 STEP 5: トラブルシューティング

### 5.1 よくある問題と解決法

**1. サービスが起動しない**
```
Error: Failed to start container
```
**解決法**:
- `SERVER_ADDR`が`0.0.0.0:10000`になっているか確認
- Environment Variablesが正しく設定されているか確認
- Renderのログで詳細エラーを確認

**2. データベース接続エラー**
```
Error: Database connection failed
```
**解決法**:
- `DATABASE_URL`に`?sslmode=require`が含まれているか確認
- Neonデータベースが稼働中か確認（一時停止状態でないか）
- CONNECTION STRINGが正しくコピーされているか確認

**3. テーブルが存在しないエラー**
```
Error: relation "tasks" does not exist
```
**解決法**:
- STEP 3のマイグレーションが正常に完了したか確認
- 以下のコマンドで再度マイグレーション実行：
  ```bash
  export DATABASE_URL="your_neon_connection_string"
  cargo run --package migration -- up
  ```

### 5.2 マイグレーション再実行
何らかの理由でマイグレーションに失敗した場合：

```bash
# 現在の状態確認
cargo run --package migration -- status

# 必要に応じてロールバック
cargo run --package migration -- down

# 再度マイグレーション実行
cargo run --package migration -- up
```

---

## 🎯 STEP 6: 完了確認

### 6.1 最終チェックリスト
- [ ] Renderサービスが「Live」ステータス
- [ ] ヘルスチェックが`OK`を返す
- [ ] マイグレーションが完了（2つのマイグレーション適用済み）
- [ ] タスク作成API が動作
- [ ] タスク一覧取得API が動作
- [ ] タスク更新・削除API が動作
- [ ] フィルタリング・ページネーション機能が動作

### 6.2 デプロイ完了情報
以下の情報を記録：

```
🎉 デプロイ完了！

🌐 API URL: https://rust-backend-api-first-lesson-latest.onrender.com
📊 Render Dashboard: https://dashboard.render.com
🗄️ Neon Dashboard: https://console.neon.tech
🐳 Container: ghcr.io/higashi-kota/rust-backend-api-first-lesson:latest

✅ Available Endpoints:
- GET  /health
- GET  /tasks
- POST /tasks
- GET  /tasks/{id}
- PATCH /tasks/{id}
- DELETE /tasks/{id}
- GET  /tasks/filter
- GET  /tasks/paginated
- POST /tasks/batch/create
- PATCH /tasks/batch/update
- POST /tasks/batch/delete
```

---

## 🚀 次のステップ・活用方法

### 監視・メンテナンス
- Renderのメトリクス機能でパフォーマンス監視
- Neonのダッシュボードでデータベース使用量確認
- 外部監視サービス（UptimeRobot等）でサービス監視

### スケーリング（将来）
- トラフィック増加時：Renderインスタンスのアップグレード
- データ量増加時：Neon有料プランへの移行

### 開発継続
- GitHub ActionsでCI/CD構築
- 認証機能の追加
- APIドキュメントの整備（Swagger/OpenAPI）

**🎉 お疲れ様でした！完全に無料でフル機能のRust APIがクラウドにデプロイされました！**

---

## 💡 重要なポイント（まとめ）

1. **Renderの無料プランの制限**：ShellやPre-Deploy Commandは有料機能
2. **ローカルマイグレーション**：必須作業として手順に組み込み
3. **環境変数の重要性**：`SERVER_ADDR=0.0.0.0:10000`は必須
4. **Neonの接続文字列**：`?sslmode=require`が必要
5. **デプロイ後の確認**：APIの全機能をテストして完了確認

この手順で確実に無料デプロイが完了します！