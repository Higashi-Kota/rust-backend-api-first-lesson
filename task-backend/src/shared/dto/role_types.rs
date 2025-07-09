use crate::error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct CreateRoleInput {
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

impl CreateRoleInput {
    pub fn validate(&self) -> AppResult<()> {
        let mut errors = Vec::new();

        // 名前バリデーション
        if self.name.trim().is_empty() {
            errors.push("Role name cannot be empty".to_string());
        } else if self.name.len() > 50 {
            errors.push("Role name must be 50 characters or less".to_string());
        } else if !self
            .name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            errors.push(
                "Role name can only contain alphanumeric characters, underscores, and hyphens"
                    .to_string(),
            );
        }

        // 表示名バリデーション
        if self.display_name.trim().is_empty() {
            errors.push("Display name cannot be empty".to_string());
        } else if self.display_name.len() > 100 {
            errors.push("Display name must be 100 characters or less".to_string());
        }

        // 説明バリデーション
        if let Some(description) = &self.description {
            if description.len() > 1000 {
                errors.push("Description must be 1000 characters or less".to_string());
            }
        }

        if !errors.is_empty() {
            return Err(AppError::ValidationErrors(errors));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct UpdateRoleInput {
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub description: Option<Option<String>>,
    pub is_active: Option<bool>,
}

impl UpdateRoleInput {
    pub fn validate(&self) -> AppResult<()> {
        let mut errors = Vec::new();

        // 名前バリデーション（提供された場合）
        if let Some(name) = &self.name {
            if name.trim().is_empty() {
                errors.push("Role name cannot be empty".to_string());
            } else if name.len() > 50 {
                errors.push("Role name must be 50 characters or less".to_string());
            } else if !name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
            {
                errors.push(
                    "Role name can only contain alphanumeric characters, underscores, and hyphens"
                        .to_string(),
                );
            }
        }

        // 表示名バリデーション（提供された場合）
        if let Some(display_name) = &self.display_name {
            if display_name.trim().is_empty() {
                errors.push("Display name cannot be empty".to_string());
            } else if display_name.len() > 100 {
                errors.push("Display name must be 100 characters or less".to_string());
            }
        }

        // 説明バリデーション（提供された場合）
        if let Some(Some(description)) = &self.description {
            if description.len() > 1000 {
                errors.push("Description must be 1000 characters or less".to_string());
            }
        }

        if !errors.is_empty() {
            return Err(AppError::ValidationErrors(errors));
        }

        Ok(())
    }
}
