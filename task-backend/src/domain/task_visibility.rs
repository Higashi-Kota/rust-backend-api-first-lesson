// src/domain/task_visibility.rs

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// タスクの可視性を表すEnum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "task_visibility")]
pub enum TaskVisibility {
    #[sea_orm(string_value = "personal")]
    #[serde(rename = "personal")]
    Personal,

    #[sea_orm(string_value = "team")]
    #[serde(rename = "team")]
    Team,

    #[sea_orm(string_value = "organization")]
    #[serde(rename = "organization")]
    Organization,
}

impl TaskVisibility {
    /// 文字列に変換
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Personal => "personal",
            Self::Team => "team",
            Self::Organization => "organization",
        }
    }
}

impl Default for TaskVisibility {
    fn default() -> Self {
        Self::Personal
    }
}

impl std::fmt::Display for TaskVisibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_visibility_as_str() {
        assert_eq!(TaskVisibility::Personal.as_str(), "personal");
        assert_eq!(TaskVisibility::Team.as_str(), "team");
        assert_eq!(TaskVisibility::Organization.as_str(), "organization");
    }

    #[test]
    fn test_task_visibility_default() {
        assert_eq!(TaskVisibility::default(), TaskVisibility::Personal);
    }

    #[test]
    fn test_task_visibility_display() {
        assert_eq!(TaskVisibility::Personal.to_string(), "personal");
        assert_eq!(TaskVisibility::Team.to_string(), "team");
        assert_eq!(TaskVisibility::Organization.to_string(), "organization");
    }
}
