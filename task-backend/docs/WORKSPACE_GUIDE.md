# ワークスペース構成への移行ガイド

このガイドでは、既存の`task-backend`プロジェクトをCargoワークスペース構成に移行する手順を説明します。

## 📁 移行前後の構成比較

### 移行前
```
rust-backend-api-first-lesson/
└── task-backend/
    ├── Cargo.toml
    ├── migration/
    │   ├── Cargo.toml
    │   └── src/
    ├── src/
    ├── docker-compose.yml
    ├── Dockerfile
    └── Makefile
```

### 移行後
```
rust-backend-api-first-lesson/
├── Cargo.toml                # ワークスペース設定
├── task-backend/
│   ├── Cargo.toml            # アプリケーション設定
│   └── src/
├── migration/                # ルートレベルに移動
│   ├── Cargo.toml
│   └── src/
├── docker-compose.yml        # ルートレベルに移動
├── Dockerfile                # ルートレベルに移動
└── Makefile                  # ルートレベルに移動
```

## 🔄 移行手順

### 1. 新規ファイルの作成

以下のファイルをルートディレクトリに作成してください：

- `Cargo.toml` (ワークスペース設定)
- `Dockerfile` (ワークスペース対応版)
- `docker-compose.yml` (ルートレベル版)
- `Makefile` (ワークスペース対応版)
- `.dockerignore` (ルートレベル版)
- `.env` (ルートレベル版)
- `rust-toolchain.toml` (ルートレベルに移動)

### 2. ディレクトリ・ファイルの移動

```bash
# migrationディレクトリをルートレベルに移動
mv task-backend/migration/ ./migration/

# ルートレベルのファイルを移動
mv task-backend/docker-compose.yml ./docker-compose.yml
mv task-backend/Dockerfile ./Dockerfile  
mv task-backend/Makefile ./Makefile
mv task-backend/.dockerignore ./.dockerignore
mv task-backend/.env ./.env
mv task-backend/rust-toolchain.toml ./rust-toolchain.toml
```

### 3. ファイルの修正

#### `task-backend/Cargo.toml`
- `migration`の依存関係のパスを`{ path = "../migration" }`に変更
- ワークスペース共通の依存関係を`workspace = true`に変更

#### `migration/Cargo.toml`
- ワークスペース共通の依存関係を`workspace = true`に変更

#### `.github/workflows/ci.yml`
- `working-directory: task-backend`を削除
- ワークスペース対応のコマンドに変更

### 4. 古いファイルの削除

```bash
# 移動済みファイルの削除
rm task-backend/docker-compose.yml
rm task-backend/Dockerfile
rm task-backend/Makefile
rm task-backend/.dockerignore
rm task-backend/.env
rm task-backend/rust-toolchain.toml

# migration関連の古いファイル削除
rm -rf task-backend/migration/
```

### 5. 動作確認

```bash
# ワークスペースの確認
cargo --version
cargo metadata --format-version 1 | jq '.workspace_members'

# ビルド確認
cargo build --workspace

# テスト実行
cargo test --workspace

# 各パッケージの個別実行確認
cargo run --package migration -- --help
cargo run --package task-backend --help
```

### 6. Docker確認

```bash
# Dockerビルド確認
docker build -t task-backend .

# Docker Compose確認
docker-compose up --build
```

## ⚠️ 注意事項

### 既存の開発環境への影響

1. **IDEの設定**
   - VS Codeなどの設定でルートディレクトリがワークスペースとして認識されることを確認
   - `rust-analyzer`がワークスペース全体を認識することを確認

2. **環境変数**
   - `.env`ファイルがルートレベルに移動するため、既存の設定を確認

3. **CI/CD**
   - GitHub Actionsのワークフローが正しく動作することを確認
   - Docker Hub等の外部サービス連携が正しく動作することを確認

### パフォーマンスへの影響

1. **ビルド時間**
   - ワークスペース構成により、共通の依存関係がキャッシュされるため、全体的なビルド時間が短縮される可能性

2. **テスト実行**
   - `cargo test --workspace`で全テストを一括実行可能
   - 個別パッケージのテストも`cargo test --package <name>`で実行可能

## 🔧 トラブルシューティング

### よくある問題と解決方法

1. **依存関係の解決エラー**
   ```bash
   cargo clean
   cargo build --workspace
   ```

2. **パス解決エラー**
   - `Cargo.toml`内の`path`指定が正しいか確認
   - 相対パスが正確に設定されているか確認

3. **Docker ビルドエラー**
   - `Dockerfile`内のCOPYパスが正しいか確認
   - `.dockerignore`の設定を確認

4. **テストの実行エラー**
   - 環境変数が正しく設定されているか確認
   - データベース接続設定を確認

## 🎯 移行後のメリット

1. **コード共有の向上**
   - 共通の依存関係を一元管理
   - 型定義やユーティリティの共有が容易

2. **ビルド効率の向上**
   - 依存関係のキャッシュ効率が向上
   - 並列ビルドの恩恵を受けやすい

3. **開発体験の向上**
   - 統一されたコマンドでワークスペース全体を操作
   - IDEサポートの向上

4. **CI/CDの効率化**
   - 一度のビルドで全コンポーネントをテスト
   - デプロイメントの一元化

## 📚 参考資料

- [Cargo Workspaces - The Rust Programming Language](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
- [Cargo Reference - Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [cargo-workspace - Managing Rust workspaces](https://github.com/pksunkara/cargo-workspaces)