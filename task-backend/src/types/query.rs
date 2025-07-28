use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

// Re-export pagination constants
pub use crate::shared::types::{DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE};

/// デフォルトページ番号
fn default_page() -> u32 {
    1
}

/// デフォルトページサイズ
fn default_per_page() -> u32 {
    DEFAULT_PAGE_SIZE
}

/// 文字列または数値からu32をデシリアライズ
fn deserialize_u32_from_string<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber {
        String(String),
        Number(u32),
    }

    match StringOrNumber::deserialize(deserializer)? {
        StringOrNumber::String(s) => s.parse::<u32>().map_err(serde::de::Error::custom),
        StringOrNumber::Number(n) => Ok(n),
    }
}

/// 統一ページネーションクエリパラメータ
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaginationQuery {
    #[serde(
        default = "default_page",
        deserialize_with = "deserialize_u32_from_string"
    )]
    pub page: u32,
    #[serde(
        default = "default_per_page",
        deserialize_with = "deserialize_u32_from_string"
    )]
    pub per_page: u32,
}

impl Default for PaginationQuery {
    fn default() -> Self {
        Self {
            page: default_page(),
            per_page: default_per_page(),
        }
    }
}

impl PaginationQuery {
    /// デフォルト値を適用してページとper_pageを取得
    pub fn get_pagination(&self) -> (i32, i32) {
        let page = self.page.max(1) as i32;
        let per_page = self.per_page.clamp(1, MAX_PAGE_SIZE) as i32;
        (page, per_page)
    }

    /// オフセットを計算
    pub fn get_offset(&self) -> i32 {
        let page = self.page.max(1);
        ((page - 1) * self.per_page) as i32
    }
}

/// 統一ソートクエリパラメータ
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SortQuery {
    pub sort_by: Option<String>,
    #[serde(default)]
    pub sort_order: SortOrder,
}

/// ソート順序
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    #[default]
    Asc,
    Desc,
}

impl SortOrder {
    pub fn as_str(&self) -> &str {
        match self {
            SortOrder::Asc => "asc",
            SortOrder::Desc => "desc",
        }
    }
}

/// 統一検索クエリトレイト
///
/// このトレイトは検索機能を持つすべてのクエリDTOに実装され、
/// 統一されたインターフェースを提供します。
///
/// ## 設計意図
/// - **インターフェースの統一**: すべての検索クエリが同じメソッドを持つことを保証
/// - **将来の拡張性**: 汎用的な検索ロジックやミドルウェアの実装時に利用可能
/// - **型安全性**: トレイト境界として使用し、コンパイル時の型チェックを実現
///
/// ## 実装状況
/// 現在、以下のDTOで実装されています：
/// - `TaskSearchQuery`
/// - `TeamSearchQuery`
/// - `OrganizationSearchQuery`
/// - `AttachmentSearchQuery`
/// - `DepartmentSearchQuery`
/// - その他
///
/// ## SeaORMとの統合方針（ハイブリッドアプローチ）
///
/// 将来的にこのトレイトを活用する際は、以下のハイブリッドアプローチを推奨：
///
/// ```rust,ignore
/// // 基本的な検索条件は汎用ヘルパーで処理
/// let mut db_query = self.apply_base_search(
///     Entity::find(),
///     query,  // SearchQueryトレイトを実装した型
///     &[Column::Name, Column::Description],  // 検索対象カラム
/// );
///
/// // エンティティ固有のフィルタは個別に実装
/// if let Some(status) = &query.status {
///     db_query = db_query.filter(Column::Status.eq(status));
/// }
/// ```
///
/// このアプローチにより：
/// - 共通的な検索処理（キーワード検索など）の再利用性を確保
/// - エンティティ固有の複雑なフィルタリングの型安全性を維持
/// - SeaORMの強力な型システムを最大限活用
///
/// ## 注意
/// 現時点ではトレイトメソッドは直接呼び出されていませんが、
/// これは将来の統一検索エンドポイントやフィルタリングロジックの
/// 実装に備えた設計です。CI要件を満たすため一時的に#[allow(dead_code)]を
/// 付与していますが、実装時には削除してください。
#[allow(dead_code)]
pub trait SearchQuery {
    /// 検索キーワードを返す
    fn search_term(&self) -> Option<&str>;

    /// フィルタ条件をキー・バリューのマップとして返す
    fn filters(&self) -> HashMap<String, String>;
}

// QueryBuilderはSeaORMの機能を使用するため削除
// SeaORMはQueryFilter、QueryOrder、PaginatorTraitなどを提供し、
// より型安全で高機能なクエリ構築が可能

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_order_default() {
        let sort = SortQuery::default();
        assert!(sort.sort_by.is_none());
        assert!(matches!(sort.sort_order, SortOrder::Asc));
    }

    // QueryBuilderテストは削除（SeaORMを使用するため）
}
