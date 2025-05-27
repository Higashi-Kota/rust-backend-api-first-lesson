# Rust テスト **ベストプラクティス** (2025‑05)

_task‑backend 向け／実運用を想定したガイド_

---

## 1️⃣ テストの種類と役割

| 層                   | 物理配置                 | 主な対象               | 推奨クレート / 技法             |
| -------------------- | ------------------------ | ---------------------- | ------------------------------- |
| **ユニット**         | `src/.. mod tests`       | 純粋関数・モデル       | `#[test]`, `rstest`, `mockall`  |
| **ドキュメント**     | `///` コメント           | 公開 API 使用例        | Rustdoc (自動)                  |
| **スナップショット** | `tests/` or `snapshots/` | JSON, HTML, CLI 出力   | `insta`                         |
| **結合**             | `tests/`                 | モジュール跨ぎ・DB/API | `tokio::test`, `testcontainers` |
| **E2E (HTTP)**       | `tests/e2e/`             | 実 HTTP サーバ         | `reqwest`, `wiremock-rs`        |
| **プロパティ**       | `tests/prop_*.rs`        | 不変則 & アルゴリズム  | `proptest`, `quickcheck`        |
| **ファズ**           | `fuzz/`                  | パーサ・バイナリ       | `cargo fuzz`                    |
| **ベンチ**           | `benches/`               | 性能                   | `criterion` (stable)            |

> **ポイント**: 単体〜結合までは _並列実行_ を前提に副作用を隔離。E2E は `#[ignore]` を付けて CI の別ジョブで走らせると高速化できる。

---

## 2️⃣ テストランナー選択

| ランナー                | 特徴                             | コマンド例                         |
| ----------------------- | -------------------------------- | ---------------------------------- |
| **`cargo test`** (標準) | 最小構成                         | `cargo test --all-features`        |
| **`cargo nextest`**     | 並列・失敗時の再実行・Junit 出力 | `cargo nextest run --all-features` |
| **`trybuild`**          | コンパイルエラーをテスト         | ビルドスクリプト内                 |

CI では **nextest + `--status-level skip`** が高速かつログが見やすい。

---

## 3️⃣ 非同期テストのコツ

```rust
#[tokio::test(start_paused = true)] // 時間を手動で進められる
async fn my_async_test() {
    time::sleep(Duration::from_secs(1)).await; // ↑実際には即完了
}
```

- `start_paused` + `tokio::time::advance()` により _deterministic_ に。
- `tracing_subscriber::fmt::init()` をテスト毎に呼ぶと二重初期化エラー → **OnceCell** で 1 回だけ初期化。

---

## 4️⃣ testcontainers + SeaORM 実践

```toml
[dev-dependencies]
# Reuse を許可すると CI が高速化 (ローカルは false 推奨)
testcontainers-modules = { version = "0.12", default-features = false, features = ["postgres", "reuse"] }
```

```rust
let image = Postgres::default()
    .with_tag("16-alpine")             // 明示タグで将来の破壊的変更を回避
    .with_env_var("POSTGRES_DB", &db_name);
let container = image.start().await?;
```

- **テスト毎に一意な DB** を作成し `search_path` を切替 → 並列でも干渉しない。
- CI の Docker 層キャッシュを有効化すると最大 2 倍高速。

---

## 5️⃣ 入力データ・フィクスチャ

- **`fake`** クレートでランダムだが再現可能な値を生成 (`Fake::fake_with_rng(&mut StdRng::seed_from_u64(42))`)。
- 大規模 JSON は `include_str!("fixtures/task.json")` で埋め込み、`serde_json::from_str`。
- スナップショットは `insta::assert_json_snapshot!` で差分レビューが楽。

---

## 6️⃣ モック戦略

| 対象       | クレート               | 使い分け                            |
| ---------- | ---------------------- | ----------------------------------- |
| 内部 trait | `mockall`, `double`    | Rust の型安全を維持したまま挿げ替え |
| 外部 HTTP  | `wiremock-rs`          | OpenAPI の例もそのまま貼れる        |
| gRPC       | `tonic` + `tower-test` | 偽サーバを立てて Channel 差替え     |

> **賢い選択**: 保存コストが高いモックは **回数制限付きの E2E テスト** に任せ、ユニットテストでは純粋ロジックを徹底。

---

## 7️⃣ パラメータ・テーブル駆動 & シード

```rust
#[rstest]
#[case::empty("", 0)]
#[case::simple("abc", 3)]
fn len_works(#[case] input: &str, #[case] expected: usize) {
    assert_eq!(input.len(), expected);
}
```

- `rstest_reuse` で共通パラメータセットを使い回すと DRY。
- プロパティテスト時は `proptest::test_runner::Config { cases: 256, failure_persistence: Some(dir!()) }` でエッジケースを永続化。

---

## 8️⃣ カバレッジ & 静的解析

| ツール            | コマンド                            | メモ                         |
| ----------------- | ----------------------------------- | ---------------------------- |
| `cargo llvm-cov`  | `cargo llvm-cov --workspace --html` | stable / accurate / HTML GUI |
| `cargo tarpaulin` | Linux 専用 / サイト統計向け         | シンプルに行単位             |
| `grcov`           | Firefox 派生                        | 古いが高速                   |

CI で 70% 以上を維持し、低下時に PR をブロック。

---

## 9️⃣ CI / CD ベストプラクティス

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-14]
    rust: [stable, nightly]

steps:
  - uses: actions/checkout@v4
  - run: rustup toolchain install ${{ matrix.rust }} --profile minimal --no-self-update
  - run: cargo nextest run --all-features
  - run: cargo clippy --all-features --all-targets -- -D warnings
  - if: matrix.os == 'ubuntu-latest'
    run: cargo llvm-cov --workspace --codecov --output-path codecov.json
```

- **次世代ランナー `nextest`** は失敗テストのみを再出力し可読性抜群。
- nightly では `-Z minimal-versions` を追加し、依存の下限バージョンでも通るか検証。

---

## 🔟 デバッグ & 開発体験

- `cargo watch -x "test --all-features"` で保存時に自動テスト。
- `just t %` みたいな **Justfile** エイリアスがあると頻繁なフィルタ実行が楽。
- VS Code は `launch.json` で `cargo test -- --exact your_test_name --nocapture` を指定するとブレークポイントが効く。

---

## 1️⃣1️⃣ まとめチェックリスト

- [ ] 単体テストは副作用レス & ミリ秒級で完了
- [ ] 結合テストは Docker レイヤキャッシュ + 並列で 1 分以内
- [ ] E2E テストは `#[ignore]` & CI 夜間ジョブ
- [ ] Lint + Format + Coverage が CI ゲートに組込まれている
- [ ] `cargo audit` / `cargo deny` で脆弱性チェックも (security)

> **Enjoy Reliable Rust!** 🦀✨

---

## 1️⃣2️⃣ 28 テストを **すべてパス** させるためのアプローチ

> **現状** > _パス_: 20 / 28
> _失敗_: 8 (`integration::api_tests::*`)
> _ハング_: 8 (60 秒超過メッセージ)

### 🗺️ ロードマップ

| ステップ                  | 目的                                       | 具体的コマンド / テクニック                                                                                             |
| ------------------------- | ------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------- |
| **① 失敗リストを固定**    | CI と同じ状態をローカルで再現              | `cargo test --all-features -- --exact integration::api_tests::test_invalid_uuid_error --nocapture`                      |
| **② ログを最大化**        | Axum / SeaORM / SQL を確認                 | `RUST_LOG=debug,cargo test … --nocapture`<br>`tracing_subscriber` 初期化を OnceCell で 1 度だけ行う                     |
| **③ DB レイヤを確認**     | マイグレーション漏れ・トランザクション競合 | _Check_: `Migrator::up` がテスト前に毎回走るか<br>_Tip_: `BEGIN; .. ROLLBACK;` でテスト毎にロールバック                 |
| **④ タイムアウト調査**    | 60 秒以上のテストを個別実行                | `cargo test test_batch_operations_endpoints -- --nocapture --test-threads 1`<br>→ ハング箇所で `dbg!()` or `println!()` |
| **⑤ API ステータス確認**  | Axum ルータのルート定義漏れ                | `cargo expand` でルータマクロ展開を確認<br>`reqwest::Client::get("/health").send()` 手動確認                            |
| **⑥ 並列干渉の切り分け**  | 同一 DB を共有しているか                   | テスト関数毎に `TestDatabase::new_with_random_schema()` を呼び、`search_path` を Schema 切替                            |
| **⑦ フィクスチャ再利用**  | 作成したタスクを流用                       | _Strategy_: 共通 helper で `create_test_user()` などを返し、重複 insert を避ける                                        |
| **⑧ Regression テスト化** | 修正後の再発防止                           | 失敗原因を最小ユニットテストに切り出し、`#[should_panic]` でガード                                                      |

### 🏹 実装上のチェックポイント

1. **`/health` が 404 → 200**

   - ルーティング階層 `task_router(app_state)` がトップレベルにマウントされているか確認。

2. **UUID バリデーション失敗**

   - `Uuid::parse_str()` のエラーパスが `BAD_REQUEST` でなく 404 を返していないか。
   - Axum の `Path<Uuid>` extractor のカスタムリジェクタで 400 を返す実装例を追加。

3. **60 秒ハング**

   - SeaORM の DB コネクションプール上限 (`max_connections`) がテスト並列数より低いと `Acquire` ハングになる。
   - `.max_connections(num_cpus::get() as u32)` に設定。

4. **DELETE → GET で 404 にならない**

   - リポジトリ層の `delete` が `returning` 句で削除件数を確認しているか。

### 🔄 短いフィードバックループを作る

```bash
# 失敗したテストだけループ実行 (変更を監視)
cargo watch -x "test integration::api_tests:: -- --exact --nocapture"
```

- テスト時間が長い場合は DB 再利用モード (`reuse` feature) と `--test-threads 1` を組合せる。

### ✅ 完全パス後に行うこと

- CI で **`cargo nextest run --retries 2`** を有効化し、flake を検知。
- `cargo llvm-cov` でカバレッジを計測し、エラー経路のテストが抜けていないか確認。
- `#[ignore]` にしていた遅い E2E テストを nightly ジョブで有効化。

> **目標**: ローカル `cargo test --all-features` が **2 分以内 / 全 28 パス**、CI でも同一結果を保証。
