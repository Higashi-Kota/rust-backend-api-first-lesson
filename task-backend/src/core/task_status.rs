// task-backend/src/domain/task_status.rs

use serde::{Deserialize, Serialize};
use std::fmt;

/// タスクの状態を表すenum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Todo,
    InProgress,
    Completed,
    Cancelled,
}

impl TaskStatus {
    /// 文字列からTaskStatusに変換
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "todo" => Some(Self::Todo),
            "in_progress" => Some(Self::InProgress),
            "completed" => Some(Self::Completed),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }

    /// TaskStatusを文字列として取得
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Todo => "todo",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }

    /// すべての有効なステータスを取得
    pub fn all() -> Vec<Self> {
        vec![
            Self::Todo,
            Self::InProgress,
            Self::Completed,
            Self::Cancelled,
        ]
    }

    /// ステータスが完了状態かチェック
    pub fn is_completed(&self) -> bool {
        matches!(self, Self::Completed)
    }

    /// ステータスがアクティブ状態かチェック（未完了）
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Todo | Self::InProgress)
    }

    /// ステータスが終了状態かチェック
    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled)
    }

    /// 有効なステータス遷移かチェック
    pub fn can_transition_to(&self, new_status: Self) -> bool {
        match (self, new_status) {
            // 同じステータスは常に有効
            (current, new) if current == &new => true,

            // Todoから他のステータスへは全て可能
            (Self::Todo, _) => true,

            // InProgressから完了・キャンセルは可能、Todoに戻すことも可能
            (Self::InProgress, Self::Completed | Self::Cancelled | Self::Todo) => true,

            // 完了・キャンセルから再開は可能
            (Self::Completed | Self::Cancelled, Self::Todo | Self::InProgress) => true,

            // その他の遷移は無効
            _ => false,
        }
    }

    /// ステータスの表示名を取得
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Todo => "To Do",
            Self::InProgress => "In Progress",
            Self::Completed => "Completed",
            Self::Cancelled => "Cancelled",
        }
    }
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self::Todo
    }
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for TaskStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str(s).ok_or_else(|| {
            format!(
                "Invalid task status: '{}'. Valid statuses are: {}",
                s,
                Self::all()
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
    }
}

// データベースとの変換用
impl From<TaskStatus> for String {
    fn from(status: TaskStatus) -> Self {
        status.as_str().to_string()
    }
}

impl TryFrom<String> for TaskStatus {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<&str> for TaskStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(TaskStatus::from_str("todo"), Some(TaskStatus::Todo));
        assert_eq!(TaskStatus::from_str("TODO"), Some(TaskStatus::Todo));
        assert_eq!(
            TaskStatus::from_str("in_progress"),
            Some(TaskStatus::InProgress)
        );
        assert_eq!(
            TaskStatus::from_str("IN_PROGRESS"),
            Some(TaskStatus::InProgress)
        );
        assert_eq!(
            TaskStatus::from_str("completed"),
            Some(TaskStatus::Completed)
        );
        assert_eq!(
            TaskStatus::from_str("COMPLETED"),
            Some(TaskStatus::Completed)
        );
        assert_eq!(
            TaskStatus::from_str("cancelled"),
            Some(TaskStatus::Cancelled)
        );
        assert_eq!(
            TaskStatus::from_str("CANCELLED"),
            Some(TaskStatus::Cancelled)
        );
        assert_eq!(TaskStatus::from_str("invalid"), None);
    }

    #[test]
    fn test_to_string() {
        assert_eq!(TaskStatus::Todo.to_string(), "todo");
        assert_eq!(TaskStatus::InProgress.to_string(), "in_progress");
        assert_eq!(TaskStatus::Completed.to_string(), "completed");
        assert_eq!(TaskStatus::Cancelled.to_string(), "cancelled");
    }

    #[test]
    fn test_status_checks() {
        assert!(TaskStatus::Completed.is_completed());
        assert!(!TaskStatus::Todo.is_completed());

        assert!(TaskStatus::Todo.is_active());
        assert!(TaskStatus::InProgress.is_active());
        assert!(!TaskStatus::Completed.is_active());
        assert!(!TaskStatus::Cancelled.is_active());

        assert!(TaskStatus::Completed.is_finished());
        assert!(TaskStatus::Cancelled.is_finished());
        assert!(!TaskStatus::Todo.is_finished());
        assert!(!TaskStatus::InProgress.is_finished());
    }

    #[test]
    fn test_transitions() {
        // 同じステータスは常に有効
        assert!(TaskStatus::Todo.can_transition_to(TaskStatus::Todo));

        // Todoから他への遷移
        assert!(TaskStatus::Todo.can_transition_to(TaskStatus::InProgress));
        assert!(TaskStatus::Todo.can_transition_to(TaskStatus::Completed));
        assert!(TaskStatus::Todo.can_transition_to(TaskStatus::Cancelled));

        // InProgressからの遷移
        assert!(TaskStatus::InProgress.can_transition_to(TaskStatus::Todo));
        assert!(TaskStatus::InProgress.can_transition_to(TaskStatus::Completed));
        assert!(TaskStatus::InProgress.can_transition_to(TaskStatus::Cancelled));

        // 完了・キャンセルからの再開
        assert!(TaskStatus::Completed.can_transition_to(TaskStatus::Todo));
        assert!(TaskStatus::Completed.can_transition_to(TaskStatus::InProgress));
        assert!(TaskStatus::Cancelled.can_transition_to(TaskStatus::Todo));
        assert!(TaskStatus::Cancelled.can_transition_to(TaskStatus::InProgress));
    }

    #[test]
    fn test_default() {
        assert_eq!(TaskStatus::default(), TaskStatus::Todo);
    }

    #[test]
    fn test_display_name() {
        assert_eq!(TaskStatus::Todo.display_name(), "To Do");
        assert_eq!(TaskStatus::InProgress.display_name(), "In Progress");
        assert_eq!(TaskStatus::Completed.display_name(), "Completed");
        assert_eq!(TaskStatus::Cancelled.display_name(), "Cancelled");
    }

    #[test]
    fn test_parse() {
        assert_eq!("todo".parse::<TaskStatus>().unwrap(), TaskStatus::Todo);
        assert_eq!(
            "in_progress".parse::<TaskStatus>().unwrap(),
            TaskStatus::InProgress
        );
        assert!("invalid".parse::<TaskStatus>().is_err());
    }

    #[test]
    fn test_conversions() {
        let status = TaskStatus::InProgress;
        let as_string: String = status.into();
        assert_eq!(as_string, "in_progress");

        let back_to_status: TaskStatus = as_string.try_into().unwrap();
        assert_eq!(back_to_status, TaskStatus::InProgress);
    }

    #[test]
    fn test_serde() {
        let status = TaskStatus::InProgress;
        let serialized = serde_json::to_string(&status).unwrap();
        assert_eq!(serialized, r#""in_progress""#);

        let deserialized: TaskStatus = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, TaskStatus::InProgress);
    }
}
