# 📦 外部クレート依存管理

Rust プロジェクトにおける外部クレート（crate）の追加・削除・管理手順をまとめたガイドです。チームや個人開発において依存関係を安全かつ効率的に保つための指針としてご活用ください。

---

## 🧰 1. 前提：cargo-edit の導入

Rust 標準ツールには `cargo add`／`cargo rm`／`cargo upgrade` が含まれていないため、補助ツールをインストールします。

```bash
cargo install cargo-edit
```

`cargo-edit` によって以下のサブコマンドが利用可能になります：

* `cargo add` で依存クレートを追加
* `cargo rm` で依存クレートを削除
* `cargo upgrade` で依存バージョンを最新化

※ 現在のサブコマンド一覧は公式ドキュメントにて確認できます ([docs.rs](https://docs.rs/crate/cargo-edit/0.3.1?utm_source=chatgpt.com))

---

## 📥 2. ライブラリの追加方法（推奨）

### ✅ 基本的な追加

```bash
cargo add <ライブラリ名>
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

編集後に以下を実行してロックファイルと依存を更新します：

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

履歴は `CHANGELOG.md` または `UPDATE_DEPENDENCIES.md` にまとめると管理しやすいです。

---

## 🔁 6. よく使うコマンドまとめ

| タスク                  | コマンド例                                        |
| -------------------- | -------------------------------------------- |
| 依存クレートを追加            | `cargo add serde`                            |
| バージョンを指定して追加         | `cargo add anyhow@1.0.80`                    |
| 機能を指定して追加            | `cargo add tokio --features full`            |
| 手動追加後に反映             | `cargo build` または `cargo update -p <ライブラリ名>` |
| 依存構成を確認              | `cargo tree`                                 |
| 未使用依存を検出             | `cargo machete`                              |
| 未使用依存を削除（cargo-edit） | `cargo rm <クレート名>`                           |

---

## 🔒 7. バージョン固定の方法（必要に応じて）

```toml
# 完全固定（アップデートを防止）
anyhow = "=1.0.80"
```

---

## 📚 8. 開発初期におすすめの基本ライブラリ（参考）

| ライブラリ名      | 用途              |
| ----------- | --------------- |
| `serde`     | JSON などのシリアライズ  |
| `tokio`     | 非同期ランタイム        |
| `anyhow`    | エラー簡易ハンドリング     |
| `thiserror` | カスタムエラー定義       |
| `axum`      | Web API フレームワーク |
| `tracing`   | 構造化ログ           |

---

## 🗡️ 9. 未使用依存の検出と削除（cargo-machete 活用）

### 9.1 インストール

```bash
cargo install cargo-machete
```

`cargo-machete` は未使用依存検出ツールで、公式README に従い `cargo machete` コマンドが利用可能になります ([docs.rs](https://docs.rs/crate/cargo-machete/latest/source/README.md?utm_source=chatgpt.com))。

### 9.2 未使用依存の検出

```bash
cargo machete
```

* 標準実行でカレントディレクトリ配下のクレートを解析し、未使用依存を表示します。
* `--with-metadata` フラグを付与すると `cargo metadata --all-features` を呼び出し、検出精度が向上します（ただし `Cargo.lock` が書き換わる場合あり） ([docs.rs](https://docs.rs/crate/cargo-machete/latest/source/README.md?utm_source=chatgpt.com))。

```bash
cargo machete --with-metadata
```

### 9.3 未使用依存の削除

手動で `Cargo.toml` を編集するか、cargo-edit の `cargo rm` で一括削除します：

```bash
cargo rm <unused-crate1> <unused-crate2> …
```

削除後は必ずビルド＆テストで動作確認：

```bash
cargo build
cargo test
```

### 9.4 誤検出の無視設定

誤検出された依存を無視したい場合、以下を `Cargo.toml` に追記します：

```toml
[package.metadata.cargo-machete]
ignored = ["prost", "ignore-this-crate"]
```

---

## ✅ 推奨運用フロー

1. **cargo-add** でライブラリを追加
2. **cargo machete** で未使用依存を検出
3. **cargo rm** で未使用依存を削除
4. **cargo build** & **cargo test** で確認
5. **cargo tree** で依存構成をチェック
6. Git に差分をコミット
7. `DEPENDENCY_MANAGEMENT.md` または `CHANGELOG.md` に履歴を記録
