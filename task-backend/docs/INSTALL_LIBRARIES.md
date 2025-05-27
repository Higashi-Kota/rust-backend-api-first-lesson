# 📦 INSTALL_LIBRARIES.md

Rust プロジェクトに外部ライブラリ（crate）をインストール（追加）する際の手順書です。チームや個人開発において、ライブラリ管理を安全かつ効率的に行うための指針とします。

---

## 🧰 1. 前提：cargo-edit の導入

Rust 公式ツールには `cargo add` が含まれていないため、まず補助ツールをインストールします。

```bash
cargo install cargo-edit
```

これにより以下のような便利なコマンドが利用可能になります：

- `cargo add`
- `cargo rm`
- `cargo upgrade`

---

## 📥 2. ライブラリの追加方法（推奨）

### ✅ 基本的な追加

```bash
cargo add ライブラリ名
```

例：

```bash
cargo add serde
```

### ✅ 特定バージョンを指定して追加

```bash
cargo add anyhow@1.0.80
```

### ✅ 機能（features）を指定して追加

```bash
cargo add tokio --features full
```

複数指定：

```bash
cargo add axum --features "macros json"
```

---

## ✍️ 3. Cargo.toml を手動で編集する場合

`cargo add` を使わず、直接 `Cargo.toml` に追記しても構いません：

```toml
[dependencies]
serde = "1.0"
tokio = { version = "1.38", features = ["full"] }
```

その後、以下でロックファイルと依存を更新します：

```bash
cargo build
```

---

## 🧪 4. インストール後の確認

### 🔧 ビルド & テスト

```bash
cargo build
cargo test
```

### 🔍 依存関係の確認

```bash
cargo tree
```

---

## 📝 5. 追加ライブラリの履歴を記録する（推奨）

```markdown
### 2025-05-12

- add `serde = "1.0.197"` for JSON serialization
- add `tokio = { version = "1.38", features = ["full"] }` for async runtime
```

履歴は `CHANGELOG.md` または `UPDATE_LIBRARIES.md` に記載することを推奨します。

---

## 🔁 6. よく使うコマンドまとめ

| タスク                   | コマンド例                                          |
| ------------------------ | --------------------------------------------------- |
| ライブラリを追加する     | `cargo add serde`                                   |
| バージョンを指定して追加 | `cargo add anyhow@1.0.80`                           |
| 機能を指定して追加する   | `cargo add tokio --features full`                   |
| 手動追加後に反映させる   | `cargo build` または `cargo update -p ライブラリ名` |
| 依存構成を確認する       | `cargo tree`                                        |

---

## 🔒 7. バージョン固定の方法（必要に応じて）

```toml
# 完全固定（アップデートを防止）
anyhow = "=1.0.80"
```

---

## 📚 8. 開発初期におすすめの基本ライブラリ（参考）

| ライブラリ名 | 用途                    |
| ------------ | ----------------------- |
| `serde`      | JSON などのシリアライズ |
| `tokio`      | 非同期ランタイム        |
| `anyhow`     | エラー簡易ハンドリング  |
| `thiserror`  | カスタムエラー定義      |
| `axum`       | Web API フレームワーク  |
| `tracing`    | 構造化ログ              |

---

## ✅ 推奨運用フロー

1. `cargo add` で追加
2. `cargo build` & `cargo test` で確認
3. `cargo tree` で依存チェック
4. Git に差分をコミット
5. `INSTALL_LIBRARIES.md` か `CHANGELOG.md` に履歴を記録

---
