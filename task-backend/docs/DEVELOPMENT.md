# ⚡ Rust API開発 - クイックコマンド集

## 🚀 **スタートアップ（毎日の開始時）**

```bash
# 一発で開発環境起動
make dev

# または段階的に
docker-compose up postgres -d
make migrate  
make run
```

## 🔍 **開発中のクイックチェック**

```bash
# コード品質チェック（1分以内）
make fmt && make clippy

# 単体テストのみ（高速）
cargo test --lib

# 統合テスト（時間がかかる）
cargo test --test integration

# 特定のテスト実行
cargo test test_create_task
```

## 🧪 **API動作確認用コマンド**

### 基本CRUD操作
```bash
# ヘルスチェック
curl http://localhost:3000/health

# タスク作成
curl -X POST http://localhost:3000/tasks \
  -H "Content-Type: application/json" \
  -d '{"title": "テストタスク", "status": "todo"}' | jq

# タスク一覧取得
curl http://localhost:3000/tasks | jq

# タスク取得（IDは上記で取得したものを使用）
curl http://localhost:3000/tasks/{TASK_ID} | jq

# タスク更新
curl -X PATCH http://localhost:3000/tasks/{TASK_ID} \
  -H "Content-Type: application/json" \
  -d '{"status": "completed"}' | jq

# タスク削除
curl -X DELETE http://localhost:3000/tasks/{TASK_ID}
```

### 高度な機能
```bash
# フィルタリング
curl "http://localhost:3000/tasks/filter?status=todo" | jq

# ページネーション
curl "http://localhost:3000/tasks/paginated?page=1&page_size=5" | jq

# 一括作成
curl -X POST http://localhost:3000/tasks/batch/create \
  -H "Content-Type: application/json" \
  -d '{
    "tasks": [
      {"title": "タスク1", "status": "todo"},
      {"title": "タスク2", "status": "todo"}
    ]
  }' | jq
```

## 🐳 **Docker関連コマンド**

```bash
# Dockerイメージビルド
make docker-build

# Docker Compose起動
make docker-run

# ログ確認
docker-compose logs -f app

# コンテナ内でのAPI確認
docker-compose exec app curl http://localhost:3000/health

# 停止 & クリーンアップ
docker-compose down -v
make clean
```

## 🔧 **トラブルシューティング**

```bash
# データベース接続確認
docker-compose exec postgres psql -U postgres -d taskdb -c "SELECT version();"

# マイグレーション状態確認
make migrate-status

# テーブル確認
docker-compose exec postgres psql -U postgres -d taskdb -c "\dt"

# 全データリセット
docker-compose down -v
docker-compose up postgres -d
make migrate
```

## 📊 **パフォーマンス & メトリクス**

```bash
# リリースビルド
make build

# パフォーマンステスト
time curl http://localhost:3000/tasks

# 複数リクエストのテスト
for i in {1..10}; do curl -s http://localhost:3000/health; done

# メモリ使用量確認
docker stats task-backend
```

## 🚢 **デプロイ準備**

```bash
# CI相当のフルチェック
make ci-check

# セキュリティ監査
make audit

# Docker Hub/GitHub Container Registryプッシュ
make ghcr-login
# git push origin main （CIが自動実行）
```

## 🎯 **開発効率化**

```bash
# ウォッチモード（自動再起動）
cargo watch -x "run --package task-backend"

# テストウォッチモード
cargo watch -x "test --package task-backend --lib"

# フォーマット監視
cargo watch -x fmt

# 複数ターミナルでの並行作業
# Terminal 1: make run
# Terminal 2: cargo watch -x "test --lib"
# Terminal 3: curl での動作確認
```

## 📝 **ワンライナー集**

```bash
# 開発環境の完全リセット
make clean && make dev

# 新機能テスト（フォーマット→テスト→実行）
make fmt && make clippy && make test && make run

# API動作確認セット
curl http://localhost:3000/health && \
curl -X POST http://localhost:3000/tasks \
  -H "Content-Type: application/json" \
  -d '{"title": "テスト", "status": "todo"}' | jq

# コンテナでの完全テスト
make docker-build && make docker-run && \
sleep 5 && curl http://localhost:3000/health

# 本番リリース準備
make ci-check && make build && make docker-build
```

---

## 💡 **開発フロー別コマンド組み合わせ**

### 🌅 **朝の開始時**
```bash
git pull origin main
make dev
```

### 🔄 **機能開発中**
```bash
# コード変更後
make fmt && make clippy
make test-app
make run
# API確認
```

### 🔍 **デバッグ時**
```bash
RUST_LOG=debug RUST_BACKTRACE=1 make run
```

### 🚀 **コミット前**
```bash
make ci-check
make docker-run
# 最終動作確認
```

### 📦 **リリース準備**
```bash
make ci-check
make build
make docker-build
git tag v0.1.0
git push origin v0.1.0
```

これらのコマンドをブックマークして、効率的に開発を進めてください！