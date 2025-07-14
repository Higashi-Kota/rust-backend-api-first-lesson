## 0. そもそも循環依存が起きると何がまずいか

* **ビルド不可能／並列化阻害** — Cargo の依存グラフは DAG（有向非循環グラフ）が前提で、クレート間にサイクルがあると解決不能です。Cargo はコンパイル単位を並列化する際にトポロジカルソートを行うため、ここで行き詰まります。([The Rust Programming Language Forum][1], [dmitryfrank.com][2])
* **コンパイル時間の悪化** — コンパイラがフィックスポイント計算を強いられ、インクリメンタルコンパイルも効きません。([The Rust Programming Language Forum][3])
* **設計の脆弱化** — 下位層が上位層に引きずられ、変更波及が止まらなくなります（安定‐不安定依存原則への違反）。

---

## 1. 共通原則（モノシリックでも多クレートでも同じ）

| 原則                                     | 背景                                         | 実践ヒント                                                              |
| -------------------------------------- | ------------------------------------------ | ------------------------------------------------------------------ |
| **① 依存方向を一方向（上→下）に固定**                 | “上位レイヤ＝意思決定／ポリシ”、 “下位レイヤ＝詳細” という層構造を守る     | Clean/Hexagonal/Onion の 3 層（`domain` → `application` → `infra`）を意識 |
| **② 依存性逆転（DIP）を徹底**                    | 下位は上位の抽象（Trait）に依存し、上位は下位の具象を知らない          | `trait Repository<T>` を `domain` に定義し、`infra::PgRepository` が実装    |
| **③ 共有は「共通クレート」か「第三のモジュール」に抽出**        | 循環の原因になる型・定数・ユーティリティを隔離                    | 例: `shared-types` クレートに `UserId`, `Money` 等を集約                     |
| **④ 検査範囲を限定する（PIMPL/Facade）**          | “何でも pub” は循環予備軍                           | **`pub(crate)`** と内部モジュールで関係を閉じ込める                                 |
| **⑤ インターフェースをメッセージ指向に寄せる**             | 依存はデータ形＝DTO だけにし、副作用はイベントで伝播               | CQRS / Event-Driven / 構造体メッセージ                                     |
| **⑥ dev-dependency のみで済む循環は dev に落とす** | Cargo は *dev-dependencies* の循環はビルドには影響させない | テスト用のモッククレートなど                                                     |

---

## 2. クレート分割時の設計パターン

> **A** = App(実行ファイル)
> **D** = Domain(純粋ロジック)
> **U** = Use-case/Application Service
> **I** = Infrastructure(外部 I/O)

```text
A ─▶ U ─▶ D ◀── traits ── I
```

* **D** は *何も* 外部に依存しない（最安定）。
* **U** は D の trait を呼ぶが、実装（I）は知らない。
* **I** が D を使いたくなったら、トレイト経由で逆依存する（DIP）。
* ビルド順序は D → I → U → A で DAG が完成。

---

## 3. 単一クレート内でのモジュール構成法

1. **トップ階層に “root.rs”** – domain trait と汎用型だけを定義。
2. **`mod infra; mod service;`** – 依存方向は下層のみ `use super::*;`。
3. **循環が出たら**

   * **統合**: 2 モジュールを 1 モジュールにまとめ “局所サイクル” に留める。
   * **抽出**: 両者が共有する型を `mod shared;` へ。([The Rust Programming Language Forum][4])

---

## 4. リファクタリング指標（数値で追う）

| 指標                                  | 取得方法                            | 目安                                       |
| ----------------------------------- | ------------------------------- | ---------------------------------------- |
| **外向依存数 / 内向依存数 (I = Instability)** | `cargo-depview -e`              | 0 ≤ I ≤ 1、上位ほど I 小さく、下位ほど大きく             |
| **循環クレート数**                         | `cargo tree -e features` でループ検出 | 常に 0                                     |
| **モジュール相互参照**                       | `cargo-modules generate graph`  | 矢印が戻っていれば要分割                             |
| **Fan-in / Fan-out**                | `cargo udeps -f json` → 集計      | Fan-out 多いのに Fan-in 少ない箇所は “安定なのに詳細” で危険 |

---

## 5. 具体例（Use-case → Trait → Impl）

```rust
// domain/src/repository.rs
pub trait UserRepository {
    fn find(&self, id: UserId) -> Option<User>;
}

// infra/src/pg_repo.rs
use domain::{UserRepository, User, UserId};
pub struct PgRepo { /* ... */ }

impl UserRepository for PgRepo {
    fn find(&self, id: UserId) -> Option<User> { /* SQL */ }
}

// app/src/main.rs
use infra::PgRepo;
use application::get_user;

fn main() {
    let repo = PgRepo::new(pool());
    let user = get_user(repo, 42);
}
```

* Domain は **Trait** のみ、Infra は **具象型**、App は **コンポジション**。
* 逆依存が生じないため、依存グラフは単純。

---

## 6. まとめチェックリスト

1. **層を決めたら矢印は絶対に逆転させない**
2. **抽象を上位に置き、実装を下位に置く（DIP）**
3. **共通型は第三の場所へ**（shared-types / shared mod）
4. **循環しそうなら “一段上” に引き上げる**（トレイト or イベント）
5. **ツールでグラフを可視化して定期レビュー**
6. **dev-dependency の循環に甘えず本番コードは DAG を死守**

この 6 点を “門番ルール” として PR チェックリストに組み込めば、暗黙知だった循環回避ポリシーをチームで共有・自動検証できます。

[1]: https://users.rust-lang.org/t/does-cargo-support-cyclic-dependencies/35666?utm_source=chatgpt.com "Does Cargo support cyclic dependencies? - help - Rust Users Forum"
[2]: https://dmitryfrank.com/articles/rust_module_system_encourages_bad_practices?utm_source=chatgpt.com "Rust Module System Encourages Poor Practices (Comparing to Go)"
[3]: https://users.rust-lang.org/t/how-to-resolve-cyclic-dependency/51387?utm_source=chatgpt.com "How to resolve cyclic dependency - Rust Users Forum"
[4]: https://users.rust-lang.org/t/resolving-cyclic-dependency-at-module-level/116990?utm_source=chatgpt.com "Resolving cyclic dependency at module level? - Rust Users Forum"
