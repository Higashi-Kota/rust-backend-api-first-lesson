# 🚀 Rust バックエンドサーバーの起動方法

## ▶️ サーバーの起動

以下のコマンドでローカルサーバーを起動します:

```sh
$ cargo run
```

## 📦 実行例ログ

```text
   Compiling task-backend v0.1.0 (/path/to/project/task-backend)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.67s
     Running `target/debug/task-backend`
2025-05-11T14:34:24.539449Z  INFO task_backend: Starting Task Backend server...
2025-05-11T14:34:24.539540Z  INFO task_backend: Configuration loaded: Config {
    database_url: "postgres://postgres:postgres@localhost:5432/task_db",
    server_addr: "0.0.0.0:3000"
}
2025-05-11T14:34:24.562521Z  INFO task_backend: Database pool created successfully.
2025-05-11T14:34:24.562658Z  INFO task_backend: Router configured. Server listening on 0.0.0.0:3000
```

## 🌐 アクセス確認

- ブラウザや API クライアント（curl/Postman など）で以下にアクセスして動作確認します:

```
http://localhost:3000
```

---

# 🧪 開発用ローカル実行ワークフローまとめ

```sh
$ cargo fmt -- --check       # フォーマットチェック
$ cargo fmt                  # 自動整形
$ cargo clippy --all-targets -- -D warnings  # 静的解析
$ cargo run                  # サーバー起動
```

---
