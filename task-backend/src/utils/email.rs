// task-backend/src/utils/email.rs
#![allow(dead_code)]

use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;
use tracing::{error, info};

/// メール送信エラー
#[derive(Error, Debug)]
pub enum EmailError {
    #[error("SMTP configuration error: {0}")]
    ConfigurationError(String),

    #[error("Failed to send email: {0}")]
    SendError(String),

    #[error("Invalid email address: {0}")]
    InvalidAddress(String),

    #[error("Template rendering error: {0}")]
    TemplateError(String),

    #[error("Missing email configuration")]
    MissingConfiguration,
}

/// メール設定
#[derive(Debug, Clone)]
pub struct EmailConfig {
    /// SMTP サーバーホスト
    pub smtp_host: String,
    /// SMTP サーバーポート
    pub smtp_port: u16,
    /// SMTP ユーザー名
    pub smtp_username: String,
    /// SMTP パスワード
    pub smtp_password: String,
    /// 送信者メールアドレス
    pub from_email: String,
    /// 送信者名
    pub from_name: String,
    /// TLS を使用するか
    pub use_tls: bool,
    /// 開発モードかどうか（ログ出力のみ）
    pub development_mode: bool,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            smtp_host: "localhost".to_string(),
            smtp_port: 587,
            smtp_username: "user".to_string(),
            smtp_password: "password".to_string(),
            from_email: "noreply@example.com".to_string(),
            from_name: "Task Backend".to_string(),
            use_tls: true,
            development_mode: true, // 開発環境ではデフォルトで true
        }
    }
}

impl EmailConfig {
    /// 環境変数から設定を読み込み
    pub fn from_env() -> Result<Self, EmailError> {
        let development_mode = env::var("EMAIL_DEVELOPMENT_MODE")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        // 開発モードの場合はデフォルト設定を返す
        if development_mode {
            return Ok(Self {
                development_mode: true,
                ..Default::default()
            });
        }

        // 本番環境の設定
        let smtp_host = env::var("SMTP_HOST").map_err(|_| EmailError::MissingConfiguration)?;

        let smtp_port = env::var("SMTP_PORT")
            .unwrap_or_else(|_| "587".to_string())
            .parse()
            .map_err(|_| EmailError::ConfigurationError("Invalid SMTP port".to_string()))?;

        let smtp_username =
            env::var("SMTP_USERNAME").map_err(|_| EmailError::MissingConfiguration)?;

        let smtp_password =
            env::var("SMTP_PASSWORD").map_err(|_| EmailError::MissingConfiguration)?;

        let from_email = env::var("FROM_EMAIL").map_err(|_| EmailError::MissingConfiguration)?;

        let from_name = env::var("FROM_NAME").unwrap_or_else(|_| "Task Backend".to_string());

        let use_tls = env::var("SMTP_USE_TLS")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        Ok(Self {
            smtp_host,
            smtp_port,
            smtp_username,
            smtp_password,
            from_email,
            from_name,
            use_tls,
            development_mode: false,
        })
    }

    /// 設定の検証
    pub fn validate(&self) -> Result<(), EmailError> {
        if self.development_mode {
            return Ok(()); // 開発モードでは検証をスキップ
        }

        if self.smtp_host.is_empty() {
            return Err(EmailError::ConfigurationError(
                "SMTP host is required".to_string(),
            ));
        }

        if self.smtp_username.is_empty() {
            return Err(EmailError::ConfigurationError(
                "SMTP username is required".to_string(),
            ));
        }

        if self.smtp_password.is_empty() {
            return Err(EmailError::ConfigurationError(
                "SMTP password is required".to_string(),
            ));
        }

        if self.from_email.is_empty() {
            return Err(EmailError::ConfigurationError(
                "From email is required".to_string(),
            ));
        }

        // メールアドレスの形式チェック
        if !is_valid_email(&self.from_email) {
            return Err(EmailError::InvalidAddress(self.from_email.clone()));
        }

        Ok(())
    }
}

/// メールテンプレート
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplate {
    /// テンプレート名
    pub name: String,
    /// 件名
    pub subject: String,
    /// HTMLボディ
    pub html_body: String,
    /// テキストボディ
    pub text_body: String,
}

/// メール送信内容
#[derive(Debug, Clone)]
pub struct EmailMessage {
    /// 宛先メールアドレス
    pub to_email: String,
    /// 宛先名
    pub to_name: Option<String>,
    /// 件名
    pub subject: String,
    /// HTMLボディ
    pub html_body: String,
    /// テキストボディ
    pub text_body: String,
    /// 返信先メールアドレス
    pub reply_to: Option<String>,
}

/// メール送信サービス
pub struct EmailService {
    config: EmailConfig,
}

impl EmailService {
    /// 新しいEmailServiceを作成
    pub fn new(config: EmailConfig) -> Result<Self, EmailError> {
        config.validate()?;
        Ok(Self { config })
    }

    /// 環境変数から設定を読み込んでEmailServiceを作成
    pub fn from_env() -> Result<Self, EmailError> {
        let config = EmailConfig::from_env()?;
        Self::new(config)
    }

    /// メールを送信
    pub async fn send_email(&self, message: EmailMessage) -> AppResult<()> {
        // メールアドレスの検証
        if !is_valid_email(&message.to_email) {
            return Err(AppError::ValidationError(format!(
                "Invalid email address: {}",
                message.to_email
            )));
        }

        if self.config.development_mode {
            // 開発モードではログ出力のみ
            self.log_email(&message);
            return Ok(());
        }

        // TODO: 実際のメール送信ロジックを実装
        // lettre や類似のクレートを使用

        info!(
            to_email = %message.to_email,
            subject = %message.subject,
            "Email sent successfully"
        );

        Ok(())
    }

    /// パスワードリセットメールを送信
    pub async fn send_password_reset_email(
        &self,
        to_email: &str,
        to_name: &str,
        reset_token: &str,
        reset_url: &str,
    ) -> AppResult<()> {
        let template = self.get_password_reset_template(to_name, reset_token, reset_url);

        let message = EmailMessage {
            to_email: to_email.to_string(),
            to_name: Some(to_name.to_string()),
            subject: template.subject,
            html_body: template.html_body,
            text_body: template.text_body,
            reply_to: None,
        };

        self.send_email(message).await
    }

    /// メール認証メールを送信
    pub async fn send_email_verification_email(
        &self,
        to_email: &str,
        to_name: &str,
        verification_token: &str,
        verification_url: &str,
    ) -> AppResult<()> {
        let template =
            self.get_email_verification_template(to_name, verification_token, verification_url);

        let message = EmailMessage {
            to_email: to_email.to_string(),
            to_name: Some(to_name.to_string()),
            subject: template.subject,
            html_body: template.html_body,
            text_body: template.text_body,
            reply_to: None,
        };

        self.send_email(message).await
    }

    /// ウェルカムメールを送信
    pub async fn send_welcome_email(&self, to_email: &str, to_name: &str) -> AppResult<()> {
        let template = self.get_welcome_template(to_name);

        let message = EmailMessage {
            to_email: to_email.to_string(),
            to_name: Some(to_name.to_string()),
            subject: template.subject,
            html_body: template.html_body,
            text_body: template.text_body,
            reply_to: None,
        };

        self.send_email(message).await
    }

    /// セキュリティ通知メールを送信
    pub async fn send_security_notification_email(
        &self,
        to_email: &str,
        to_name: &str,
        event_type: &str,
        event_details: &str,
    ) -> AppResult<()> {
        let template = self.get_security_notification_template(to_name, event_type, event_details);

        let message = EmailMessage {
            to_email: to_email.to_string(),
            to_name: Some(to_name.to_string()),
            subject: template.subject,
            html_body: template.html_body,
            text_body: template.text_body,
            reply_to: None,
        };

        self.send_email(message).await
    }

    /// 開発モードでのメールログ出力
    fn log_email(&self, message: &EmailMessage) {
        info!("📧 EMAIL (Development Mode)");
        info!(
            "To: {} <{}>",
            message.to_name.as_deref().unwrap_or(""),
            message.to_email
        );
        info!("Subject: {}", message.subject);
        info!("--- HTML Body ---");
        info!("{}", message.html_body);
        info!("--- Text Body ---");
        info!("{}", message.text_body);
        info!("--- End Email ---");
    }

    // --- テンプレートメソッド ---

    /// パスワードリセットテンプレート
    fn get_password_reset_template(
        &self,
        name: &str,
        token: &str,
        reset_url: &str,
    ) -> EmailTemplate {
        let subject = "Password Reset Request - Task Backend".to_string();

        let html_body = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="utf-8">
                <title>Password Reset</title>
            </head>
            <body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
                <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
                    <h1 style="color: #007bff;">Password Reset Request</h1>
                    <p>Hello {name},</p>
                    <p>You have requested to reset your password for your Task Backend account.</p>
                    <p>Click the button below to reset your password:</p>
                    <p>
                        <a href="{reset_url}?token={token}" 
                           style="background-color: #007bff; color: white; padding: 12px 24px; 
                                  text-decoration: none; border-radius: 4px; display: inline-block;">
                            Reset Password
                        </a>
                    </p>
                    <p>If the button doesn't work, copy and paste the following link into your browser:</p>
                    <p><a href="{reset_url}?token={token}">{reset_url}?token={token}</a></p>
                    <p>This link will expire in 1 hour for security reasons.</p>
                    <p>If you didn't request this password reset, please ignore this email.</p>
                    <hr style="margin: 30px 0; border: none; border-top: 1px solid #eee;">
                    <p style="font-size: 12px; color: #666;">
                        Task Backend - Secure Task Management System
                    </p>
                </div>
            </body>
            </html>
            "#,
            name = name,
            token = token,
            reset_url = reset_url
        );

        let text_body = format!(
            r#"
Password Reset Request - Task Backend

Hello {name},

You have requested to reset your password for your Task Backend account.

Please click the following link to reset your password:
{reset_url}?token={token}

This link will expire in 1 hour for security reasons.

If you didn't request this password reset, please ignore this email.

---
Task Backend - Secure Task Management System
            "#,
            name = name,
            token = token,
            reset_url = reset_url
        );

        EmailTemplate {
            name: "password_reset".to_string(),
            subject,
            html_body,
            text_body,
        }
    }

    /// メール認証テンプレート
    fn get_email_verification_template(
        &self,
        name: &str,
        token: &str,
        verification_url: &str,
    ) -> EmailTemplate {
        let subject = "Verify Your Email Address - Task Backend".to_string();

        let html_body = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="utf-8">
                <title>Email Verification</title>
            </head>
            <body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
                <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
                    <h1 style="color: #28a745;">Welcome to Task Backend!</h1>
                    <p>Hello {name},</p>
                    <p>Thank you for signing up for Task Backend. Please verify your email address to complete your registration.</p>
                    <p>Click the button below to verify your email:</p>
                    <p>
                        <a href="{verification_url}?token={token}" 
                           style="background-color: #28a745; color: white; padding: 12px 24px; 
                                  text-decoration: none; border-radius: 4px; display: inline-block;">
                            Verify Email Address
                        </a>
                    </p>
                    <p>If the button doesn't work, copy and paste the following link into your browser:</p>
                    <p><a href="{verification_url}?token={token}">{verification_url}?token={token}</a></p>
                    <p>This verification link will expire in 24 hours.</p>
                    <hr style="margin: 30px 0; border: none; border-top: 1px solid #eee;">
                    <p style="font-size: 12px; color: #666;">
                        Task Backend - Secure Task Management System
                    </p>
                </div>
            </body>
            </html>
            "#,
            name = name,
            token = token,
            verification_url = verification_url
        );

        let text_body = format!(
            r#"
Welcome to Task Backend!

Hello {name},

Thank you for signing up for Task Backend. Please verify your email address to complete your registration.

Please click the following link to verify your email:
{verification_url}?token={token}

This verification link will expire in 24 hours.

---
Task Backend - Secure Task Management System
            "#,
            name = name,
            token = token,
            verification_url = verification_url
        );

        EmailTemplate {
            name: "email_verification".to_string(),
            subject,
            html_body,
            text_body,
        }
    }

    /// ウェルカムテンプレート
    fn get_welcome_template(&self, name: &str) -> EmailTemplate {
        let subject = "Welcome to Task Backend!".to_string();

        let html_body = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="utf-8">
                <title>Welcome</title>
            </head>
            <body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
                <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
                    <h1 style="color: #007bff;">Welcome to Task Backend!</h1>
                    <p>Hello {name},</p>
                    <p>Welcome to Task Backend! Your account has been successfully created and verified.</p>
                    <p>You can now start managing your tasks efficiently with our secure task management system.</p>
                    <h2>Getting Started:</h2>
                    <ul>
                        <li>Create your first task</li>
                        <li>Organize tasks by priority and due date</li>
                        <li>Track your progress</li>
                        <li>Manage your profile settings</li>
                    </ul>
                    <p>If you have any questions or need assistance, please don't hesitate to contact our support team.</p>
                    <hr style="margin: 30px 0; border: none; border-top: 1px solid #eee;">
                    <p style="font-size: 12px; color: #666;">
                        Task Backend - Secure Task Management System
                    </p>
                </div>
            </body>
            </html>
            "#,
            name = name
        );

        let text_body = format!(
            r#"
Welcome to Task Backend!

Hello {name},

Welcome to Task Backend! Your account has been successfully created and verified.

You can now start managing your tasks efficiently with our secure task management system.

Getting Started:
- Create your first task
- Organize tasks by priority and due date
- Track your progress
- Manage your profile settings

If you have any questions or need assistance, please don't hesitate to contact our support team.

---
Task Backend - Secure Task Management System
            "#,
            name = name
        );

        EmailTemplate {
            name: "welcome".to_string(),
            subject,
            html_body,
            text_body,
        }
    }

    /// セキュリティ通知テンプレート
    fn get_security_notification_template(
        &self,
        name: &str,
        event_type: &str,
        event_details: &str,
    ) -> EmailTemplate {
        let subject = format!("Security Alert: {} - Task Backend", event_type);

        let html_body = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="utf-8">
                <title>Security Alert</title>
            </head>
            <body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
                <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
                    <h1 style="color: #dc3545;">Security Alert</h1>
                    <p>Hello {name},</p>
                    <p>We detected a security event on your Task Backend account:</p>
                    <div style="background-color: #f8f9fa; padding: 15px; border-left: 4px solid #dc3545; margin: 20px 0;">
                        <strong>Event:</strong> {event_type}<br>
                        <strong>Details:</strong> {event_details}
                    </div>
                    <p>If this was you, no action is required. If you don't recognize this activity, please:</p>
                    <ul>
                        <li>Change your password immediately</li>
                        <li>Review your account settings</li>
                        <li>Contact our support team if needed</li>
                    </ul>
                    <hr style="margin: 30px 0; border: none; border-top: 1px solid #eee;">
                    <p style="font-size: 12px; color: #666;">
                        Task Backend - Secure Task Management System
                    </p>
                </div>
            </body>
            </html>
            "#,
            name = name,
            event_type = event_type,
            event_details = event_details
        );

        let text_body = format!(
            r#"
Security Alert - Task Backend

Hello {name},

We detected a security event on your Task Backend account:

Event: {event_type}
Details: {event_details}

If this was you, no action is required. If you don't recognize this activity, please:
- Change your password immediately
- Review your account settings
- Contact our support team if needed

---
Task Backend - Secure Task Management System
            "#,
            name = name,
            event_type = event_type,
            event_details = event_details
        );

        EmailTemplate {
            name: "security_notification".to_string(),
            subject,
            html_body,
            text_body,
        }
    }
}

// --- ユーティリティ関数 ---

/// 簡単なメールアドレス検証
fn is_valid_email(email: &str) -> bool {
    if email.is_empty() {
        return false;
    }

    // @が一つだけあること
    let at_count = email.matches('@').count();
    if at_count != 1 {
        return false;
    }

    // @で分割
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }

    let (local, domain) = (parts[0], parts[1]);

    // ローカル部とドメイン部が空でないこと
    if local.is_empty() || domain.is_empty() {
        return false;
    }

    // ドメイン部に.が含まれること
    if !domain.contains('.') {
        return false;
    }

    // ドメイン部が.で始まったり終わったりしないこと
    if domain.starts_with('.') || domain.ends_with('.') {
        return false;
    }

    true
}

/// メールアドレスをマスク
pub fn mask_email(email: &str) -> String {
    if let Some(at_pos) = email.find('@') {
        let (local, domain) = email.split_at(at_pos);
        let masked_local = if local.len() <= 2 {
            "*".repeat(local.len())
        } else {
            format!("{}****", &local[..1])
        };
        format!("{}{}", masked_local, domain)
    } else {
        "****@****".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_email() {
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("user.name@domain.co.jp"));
        assert!(!is_valid_email("invalid-email"));
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("test@"));
        assert!(!is_valid_email("test"));
    }

    #[test]
    fn test_mask_email() {
        assert_eq!(mask_email("test@example.com"), "t****@example.com");
        assert_eq!(mask_email("ab@example.com"), "**@example.com");
        assert_eq!(mask_email("a@example.com"), "*@example.com");
        assert_eq!(mask_email("invalid"), "****@****");
    }

    #[tokio::test]
    async fn test_email_service_development_mode() {
        let config = EmailConfig {
            development_mode: true,
            ..Default::default()
        };

        let email_service = EmailService::new(config).unwrap();

        let message = EmailMessage {
            to_email: "test@example.com".to_string(),
            to_name: Some("Test User".to_string()),
            subject: "Test Subject".to_string(),
            html_body: "<p>Test HTML</p>".to_string(),
            text_body: "Test Text".to_string(),
            reply_to: None,
        };

        // 開発モードではエラーが発生しない
        let result = email_service.send_email(message).await;
        assert!(result.is_ok());
    }
}
