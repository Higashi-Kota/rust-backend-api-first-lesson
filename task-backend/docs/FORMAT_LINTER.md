# 🛠 フォーマット & リンタのローカル実行手順（Rust）

- まずコードを整形し、その後 Clippy で検査する流れが一般的です。

## ✅ フォーマットチェック（`rustfmt`）

コードが正しく整形されているかをチェック（修正はしない）:

```sh
$ cargo fmt -- --check
```

- 差分があるとエラーになります。
- CI で自動整形を防ぎたい場合に有効です。

## 🧹 フォーマット実行（整形）

コードを自動で整形するには以下を実行します:

```sh
$ cargo fmt
```

- プロジェクト全体の Rust ファイルが対象になります。

## 🔍 リンター実行（`clippy`）

Clippy による静的解析を実行し、警告をエラーとして扱います:

```sh
$ cargo clippy --all-targets -- -D warnings
```

- `--all-targets`: テストやビルドスクリプトも含めて解析対象にします。
- `-D warnings`: すべての警告をエラーとして扱います（CI 向け推奨）。

---

## 🔁 開発時のおすすめワークフロー

```sh
$ cargo fmt && cargo clippy --all-targets -- -D warnings
```
