> [!CAUTION]
> GitHub usernameは小文字
>

## 🚀 **方法1: Makefileのコマンドを使用（推奨）**

### ステップ1: GHCRからイメージをプル

```bash
make docker-pull-ghcr
```

プロンプトが表示されたら：
- GitHub username: `Higashi-Kota`
- Repository name: `rust-backend-api-first-lesson`

### ステップ2: PostgreSQLを起動

```bash
docker-compose up postgres -d
```

### ステップ3: GHCRイメージで実行

```bash
make run-ghcr
```

同様にプロンプトで入力：
- GitHub username: `Higashi-Kota`
- Repository name: `rust-backend-api-first-lesson`

---

## 🚀 **方法2: 直接コマンドで実行**

### ステップ1: イメージをプル

```bash
# 最新版をプル
docker pull ghcr.io/higashi-kota/rust-backend-api-first-lesson:latest

# または特定のタグをプル
docker pull ghcr.io/higashi-kota/rust-backend-api-first-lesson:main
```

### ステップ2: PostgreSQLを起動

```bash
docker-compose up postgres -d
```

### ステップ3: コンテナを実行

```bash
# PostgreSQLが起動するまで少し待機
sleep 5

# マイグレーション実行
docker run --rm --network host \
  -e DATABASE_URL=postgres://postgres:password@localhost:5432/taskdb \
  ghcr.io/higashi-kota/rust-backend-api-first-lesson:latest \
  migration up

# アプリケーション実行
docker run -d --name task-backend-ghcr \
  --network host \
  -e DATABASE_URL=postgres://postgres:password@localhost:5432/taskdb \
  -e SERVER_ADDR=0.0.0.0:3000 \
  -e RUST_LOG=info \
  ghcr.io/higashi-kota/rust-backend-api-first-lesson:latest
```

---

## 🔍 **動作確認**

### API動作確認

```bash
# ヘルスチェック
curl http://localhost:3000/health

# タスク作成
curl -X POST http://localhost:3000/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "title": "GHCR Test Task",
    "description": "Testing GHCR image",
    "status": "todo"
  }' | jq

# タスク一覧取得
curl http://localhost:3000/tasks | jq
```

### コンテナの状態確認

```bash
# コンテナログ確認
docker logs task-backend-ghcr

# コンテナ一覧
docker ps

# リソース使用量
docker stats task-backend-ghcr
```

---

## 🐳 **方法3: Docker Composeを使用（最も簡単）**

`docker-compose.yml`を以下のように編集：

```yaml
services:
  app:
    # ローカルビルドをコメントアウト
    # build:
    #   context: .
    #   dockerfile: Dockerfile

    # GHCRイメージを使用
    image: ghcr.io/higashi-kota/rust-backend-api-first-lesson:latest
    
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgres://postgres:password@postgres:5432/taskdb
      - SERVER_ADDR=0.0.0.0:3000
      - RUST_LOG=info
    depends_on:
      postgres:
        condition: service_healthy
      migration:
        condition: service_completed_successfully
    restart: unless-stopped
    networks:
      - app-network

  # migrationサービスも同様にGHCRイメージに変更
  migration:
    image: ghcr.io/higashi-kota/rust-backend-api-first-lesson:latest
    environment:
      - DATABASE_URL=postgres://postgres:password@postgres:5432/taskdb
    depends_on:
      postgres:
        condition: service_healthy
    command: ["migration", "up"]
    networks:
      - app-network

  # PostgreSQLはそのまま
  postgres:
    # ... 既存の設定
```

そして実行：

```bash
docker-compose up
```

---

## 🛠️ **トラブルシューティング**

### プライベートリポジトリの場合

もしリポジトリがプライベートな場合は、先にログインが必要です：

```bash
# Personal Access Tokenでログイン
echo $GITHUB_TOKEN | docker login ghcr.io -u Higashi-Kota --password-stdin

# または対話式でログイン
make ghcr-login
```

### イメージが見つからない場合

```bash
# 利用可能なタグを確認
docker search ghcr.io/higashi-kota/rust-backend-api-first-lesson

# または直接GitHubで確認
# https://github.com/Higashi-Kota/rust-backend-api-first-lesson/pkgs/container/rust-backend-api-first-lesson
```

### ネットワーク接続の問題

```bash
# ホストネットワークを使用
docker run --network host \
  -e DATABASE_URL=postgres://postgres:password@localhost:5432/taskdb \
  ghcr.io/higashi-kota/rust-backend-api-first-lesson:latest
```

---

これで、CI/CDでビルドされたイメージがローカルでも正しく動作することを確認できます！