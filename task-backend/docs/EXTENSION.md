# ✅ Rust 専用拡張機能

| 拡張名               | 概要                                                                                                                        |
| -------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| **rust-analyzer**    | Rust 開発で必須。型補完、ジャンプ、リント、フォーマット、ドキュメント表示など多機能な LSP（Language Server Protocol）実装。 |
| **crates-io**        | `Cargo.toml` ファイルでクレート（ライブラリ）のバージョン補完、最新バージョンチェック、ドキュメントリンク参照が可能。       |
| **Even Better TOML** | TOML 1.0 対応のハイライト、補完、フォーマット、構文検証を提供する最新の TOML 拡張機能（旧 Better TOML 非推奨）。            |

---

# ✨ 補足：よくある Rust 初心者向け設定

- `rust-analyzer`の設定ファイル（`.vscode/settings.json`）に以下を追加すると便利です：

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "rust-lang.rust-analyzer"
}
```

---
