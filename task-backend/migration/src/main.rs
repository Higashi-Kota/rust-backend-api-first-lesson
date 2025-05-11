// migration/src/main.rs

// lib.rs で pub にした Migrator をインポート
use migration::Migrator;
// sea_orm_migration の cli モジュールを使うために prelude をインポート
use sea_orm_migration::prelude::*;

// マイグレーションの実行には非同期ランタイムが必要
// migration/Cargo.toml で async-std を依存関係に入れているので、それを使用
#[async_std::main]
async fn main() {
    // sea-orm-migration が提供するコマンドラインインターフェースを実行
    cli::run_cli(Migrator).await;
}