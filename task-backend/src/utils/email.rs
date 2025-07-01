// task-backend/src/utils/email.rs

use crate::error::{AppError, AppResult};
use lettre::message::{header, MultiPart, SinglePart};
use lettre::{Message, SmtpTransport, Transport};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

/// メールプロバイダーの種類
#[derive(Debug, Clone, PartialEq)]
pub enum EmailProvider {
    /// 開発モード（コンソール出力）
    Development,
    /// MailHog（開発環境SMTP）
    MailHog,
    /// Mailgun（本番環境API）
    Mailgun,
}

/// メール設定
#[derive(Debug, Clone)]
pub struct EmailConfig {
    /// メールプロバイダー
    pub provider: EmailProvider,
    /// SMTP サーバーホスト（MailHog用）
    pub smtp_host: String,
    /// SMTP サーバーポート（MailHog用）
    pub smtp_port: u16,
    /// 送信者メールアドレス
    pub from_email: String,
    /// 送信者名
    pub from_name: String,
    /// Mailgun API キー
    pub mailgun_api_key: Option<String>,
    /// Mailgun ドメイン
    pub mailgun_domain: Option<String>,
    /// 開発モードかどうか（コンソール出力のみ）
    pub development_mode: bool,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            provider: EmailProvider::Development,
            smtp_host: "localhost".to_string(),
            smtp_port: 1025, // MailHogのデフォルトポート
            from_email: "noreply@example.com".to_string(),
            from_name: "Task Backend".to_string(),
            mailgun_api_key: None,
            mailgun_domain: None,
            development_mode: true, // 開発環境ではデフォルトで true
        }
    }
}

impl EmailConfig {
    /// 環境変数から設定を読み込む
    pub fn from_env() -> Result<Self, crate::error::AppError> {
        use std::env;

        // プロバイダーを決定
        let provider = match env::var("EMAIL_PROVIDER").as_deref() {
            Ok("mailgun") => EmailProvider::Mailgun,
            Ok("mailhog") => EmailProvider::MailHog,
            Ok("development") => EmailProvider::Development,
            Err(_) => {
                // EMAIL_PROVIDERが未設定の場合は環境に基づいて決定
                let environment = env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());
                if environment == "development" {
                    EmailProvider::Development
                } else {
                    EmailProvider::MailHog
                }
            },
            Ok(other) => {
                return Err(crate::error::AppError::InternalServerError(format!(
                    "Unknown email provider: {}",
                    other
                )))
            }
        };

        // development_modeはproviderがDevelopmentの場合のみtrueに
        let development_mode = matches!(provider, EmailProvider::Development);

        // メール設定を構築
        let config = Self {
            provider: provider.clone(),
            smtp_host: env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_string()),
            smtp_port: env::var("SMTP_PORT")
                .unwrap_or_else(|_| "1025".to_string())
                .parse()
                .map_err(|_| {
                    crate::error::AppError::InternalServerError("Invalid SMTP port".to_string())
                })?,
            from_email: env::var("FROM_EMAIL")
                .unwrap_or_else(|_| "noreply@example.com".to_string()),
            from_name: env::var("FROM_NAME").unwrap_or_else(|_| "Task Backend".to_string()),
            mailgun_api_key: env::var("MAILGUN_API_KEY").ok(),
            mailgun_domain: env::var("MAILGUN_DOMAIN").ok(),
            development_mode,
        };

        // 設定を検証
        config.validate()?;

        Ok(config)
    }

    /// 設定の検証
    pub fn validate(&self) -> Result<(), crate::error::AppError> {
        match &self.provider {
            EmailProvider::Development => Ok(()),
            EmailProvider::MailHog => {
                if self.smtp_host.is_empty() {
                    return Err(crate::error::AppError::InternalServerError(
                        "SMTP host is required for MailHog".to_string(),
                    ));
                }
                if self.from_email.is_empty() {
                    return Err(crate::error::AppError::InternalServerError(
                        "From email is required".to_string(),
                    ));
                }
                if !is_valid_email(&self.from_email) {
                    return Err(crate::error::AppError::ValidationError(format!(
                        "Invalid from email address: {}",
                        self.from_email
                    )));
                }
                Ok(())
            }
            EmailProvider::Mailgun => {
                if self.mailgun_api_key.is_none() || self.mailgun_domain.is_none() {
                    return Err(crate::error::AppError::InternalServerError(
                        "Mailgun API key and domain are required".to_string(),
                    ));
                }
                if self.from_email.is_empty() {
                    return Err(crate::error::AppError::InternalServerError(
                        "From email is required".to_string(),
                    ));
                }
                if !is_valid_email(&self.from_email) {
                    return Err(crate::error::AppError::ValidationError(format!(
                        "Invalid from email address: {}",
                        self.from_email
                    )));
                }
                Ok(())
            }
        }
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
    pub fn new(config: EmailConfig) -> Result<Self, crate::error::AppError> {
        config.validate()?;
        Ok(Self { config })
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

        // プロバイダーに応じたメール送信
        let result = match &self.config.provider {
            EmailProvider::Development => {
                self.log_email(&message);
                Ok(())
            }
            EmailProvider::MailHog => self.send_mailhog_email(&message).await,
            EmailProvider::Mailgun => self.send_mailgun_email(&message).await,
        };

        if result.is_ok() {
            info!(
                to_email = %message.to_email,
                subject = %message.subject,
                provider = ?self.config.provider,
                "Email sent successfully"
            );
        }

        result
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

    /// アカウント削除確認メールを送信
    pub async fn send_account_deletion_confirmation_email(
        &self,
        to_email: &str,
        to_name: &str,
    ) -> AppResult<()> {
        let template = self.get_account_deletion_confirmation_template(to_name);

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

    /// チーム招待メールを送信
    pub async fn send_team_invitation_email(
        &self,
        to_email: &str,
        to_name: &str,
        team_name: &str,
        inviter_name: &str,
        invitation_url: &str,
    ) -> AppResult<()> {
        let template =
            self.get_team_invitation_template(to_name, team_name, inviter_name, invitation_url);

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

    /// サブスクリプション変更メールを送信
    pub async fn send_subscription_change_email(
        &self,
        to_email: &str,
        to_name: &str,
        old_tier: &str,
        new_tier: &str,
    ) -> AppResult<()> {
        let template = self.get_subscription_change_template(to_name, old_tier, new_tier);

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

    /// MailHog経由でメールを送信
    async fn send_mailhog_email(&self, message: &EmailMessage) -> AppResult<()> {
        info!(
            "Connecting to MailHog at {}:{}",
            self.config.smtp_host, self.config.smtp_port
        );

        // MailHogはSMTPサーバーなので認証なしでSMTP送信
        let smtp_transport = SmtpTransport::builder_dangerous(&self.config.smtp_host)
            .port(self.config.smtp_port)
            .build();

        // メールメッセージを構築
        let email_message = self.build_lettre_message(message)?;

        // メール送信
        match smtp_transport.send(&email_message) {
            Ok(_) => {
                info!(
                    to_email = %message.to_email,
                    subject = %message.subject,
                    "MailHog email sent successfully"
                );
                Ok(())
            }
            Err(e) => {
                error!(
                    to_email = %message.to_email,
                    subject = %message.subject,
                    error = %e,
                    "Failed to send MailHog email"
                );
                Err(AppError::InternalServerError(format!(
                    "Failed to send email via MailHog: {}",
                    e
                )))
            }
        }
    }

    /// Mailgun経由でメールを送信
    async fn send_mailgun_email(&self, message: &EmailMessage) -> AppResult<()> {
        let api_key = self.config.mailgun_api_key.as_ref().ok_or_else(|| {
            AppError::InternalServerError("Mailgun API key not configured".to_string())
        })?;

        let domain = self.config.mailgun_domain.as_ref().ok_or_else(|| {
            AppError::InternalServerError("Mailgun domain not configured".to_string())
        })?;

        let from = format!("{} <{}>", self.config.from_name, self.config.from_email);
        let url = format!("https://api.mailgun.net/v3/{}/messages", domain);

        let client = Client::new();

        let form = reqwest::multipart::Form::new()
            .text("from", from)
            .text("to", message.to_email.clone())
            .text("subject", message.subject.clone())
            .text("html", message.html_body.clone())
            .text("text", message.text_body.clone());

        match client
            .post(&url)
            .basic_auth("api", Some(api_key))
            .multipart(form)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    info!(
                        to_email = %message.to_email,
                        subject = %message.subject,
                        "Mailgun email sent successfully"
                    );
                    Ok(())
                } else {
                    let status = response.status();
                    let error_text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    error!(
                        to_email = %message.to_email,
                        subject = %message.subject,
                        status = %status,
                        error = %error_text,
                        "Mailgun API error"
                    );
                    Err(AppError::InternalServerError(format!(
                        "Mailgun API error {}: {}",
                        status, error_text
                    )))
                }
            }
            Err(e) => {
                error!(
                    to_email = %message.to_email,
                    subject = %message.subject,
                    error = %e,
                    "Failed to send Mailgun email"
                );
                Err(AppError::InternalServerError(format!(
                    "Failed to send email via Mailgun: {}",
                    e
                )))
            }
        }
    }

    /// lettreのメッセージを構築
    fn build_lettre_message(&self, message: &EmailMessage) -> AppResult<Message> {
        // 送信者のメールボックスを構築
        let from = format!("{} <{}>", self.config.from_name, self.config.from_email)
            .parse()
            .map_err(|e| AppError::ValidationError(format!("Invalid from email address: {}", e)))?;

        // 受信者のメールボックスを構築
        let to = if let Some(to_name) = &message.to_name {
            format!("{} <{}>", to_name, message.to_email)
        } else {
            message.to_email.clone()
        }
        .parse()
        .map_err(|e| AppError::ValidationError(format!("Invalid to email address: {}", e)))?;

        // メッセージビルダーを開始
        let mut email_builder = Message::builder()
            .from(from)
            .to(to)
            .subject(&message.subject);

        // 返信先が指定されている場合は追加
        if let Some(reply_to) = &message.reply_to {
            let reply_to_mailbox = reply_to.parse().map_err(|e| {
                AppError::ValidationError(format!("Invalid reply-to email address: {}", e))
            })?;
            email_builder = email_builder.reply_to(reply_to_mailbox);
        }

        // HTMLとテキストの両方を含むマルチパートメッセージを構築
        let multipart = MultiPart::alternative()
            .singlepart(
                SinglePart::builder()
                    .header(header::ContentType::TEXT_PLAIN)
                    .body(message.text_body.clone()),
            )
            .singlepart(
                SinglePart::builder()
                    .header(header::ContentType::TEXT_HTML)
                    .body(message.html_body.clone()),
            );

        // 最終的なメッセージを構築
        email_builder.multipart(multipart).map_err(|e| {
            AppError::InternalServerError(format!("Failed to build email message: {}", e))
        })
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

    /// アカウント削除確認テンプレート
    fn get_account_deletion_confirmation_template(&self, name: &str) -> EmailTemplate {
        let subject = "Account Deletion Confirmation - Task Backend".to_string();

        let html_body = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="utf-8">
                <title>Account Deletion Confirmation</title>
            </head>
            <body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
                <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
                    <h1 style="color: #dc3545;">Account Deletion Confirmation</h1>
                    <p>Hello {name},</p>
                    <p>We are writing to confirm that your Task Backend account has been successfully deleted.</p>
                    <div style="background-color: #f8f9fa; padding: 15px; border-left: 4px solid #dc3545; margin: 20px 0;">
                        <strong>Account Status:</strong> Permanently Deleted<br>
                        <strong>Deletion Time:</strong> {deletion_time}
                    </div>
                    <p><strong>What has been deleted:</strong></p>
                    <ul>
                        <li>Your user profile and account information</li>
                        <li>All your tasks and associated data</li>
                        <li>Your authentication tokens and sessions</li>
                        <li>All password reset tokens</li>
                        <li>Any subscription history records</li>
                    </ul>
                    <p><strong>Important Notes:</strong></p>
                    <ul>
                        <li>This action is permanent and cannot be undone</li>
                        <li>You will no longer be able to access Task Backend with this email</li>
                        <li>If you wish to use Task Backend again, you will need to create a new account</li>
                        <li>This deletion was processed at your request with password verification</li>
                    </ul>
                    <p>If you did not request this account deletion or believe this was done in error, please contact our support team immediately.</p>
                    <hr style="margin: 30px 0; border: none; border-top: 1px solid #eee;">
                    <p style="font-size: 12px; color: #666;">
                        Task Backend - Secure Task Management System<br>
                        This is an automated message confirming your account deletion.
                    </p>
                </div>
            </body>
            </html>
            "#,
            name = name,
            deletion_time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        let text_body = format!(
            r#"
Account Deletion Confirmation - Task Backend

Hello {name},

We are writing to confirm that your Task Backend account has been successfully deleted.

Account Status: Permanently Deleted
Deletion Time: {deletion_time}

What has been deleted:
- Your user profile and account information
- All your tasks and associated data
- Your authentication tokens and sessions
- All password reset tokens
- Any subscription history records

Important Notes:
- This action is permanent and cannot be undone
- You will no longer be able to access Task Backend with this email
- If you wish to use Task Backend again, you will need to create a new account
- This deletion was processed at your request with password verification

If you did not request this account deletion or believe this was done in error, please contact our support team immediately.

---
Task Backend - Secure Task Management System
This is an automated message confirming your account deletion.
            "#,
            name = name,
            deletion_time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        EmailTemplate {
            name: "account_deletion_confirmation".to_string(),
            subject,
            html_body,
            text_body,
        }
    }

    /// チーム招待テンプレート
    fn get_team_invitation_template(
        &self,
        name: &str,
        team_name: &str,
        inviter_name: &str,
        invitation_url: &str,
    ) -> EmailTemplate {
        let subject = format!("You're invited to join {} team - Task Backend", team_name);

        let html_body = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="utf-8">
                <title>Team Invitation</title>
            </head>
            <body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
                <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
                    <h1 style="color: #007bff;">You're Invited to Join a Team!</h1>
                    <p>Hello {name},</p>
                    <p><strong>{inviter_name}</strong> has invited you to join the <strong>"{team_name}"</strong> team on Task Backend.</p>
                    
                    <div style="background-color: #f8f9fa; padding: 15px; border-left: 4px solid #007bff; margin: 20px 0;">
                        <strong>Team:</strong> {team_name}<br>
                        <strong>Invited by:</strong> {inviter_name}<br>
                        <strong>Platform:</strong> Task Backend
                    </div>
                    
                    <p>By joining this team, you'll be able to:</p>
                    <ul>
                        <li>Collaborate on shared tasks and projects</li>
                        <li>Access team-specific features and data</li>
                        <li>Work together with team members</li>
                        <li>Share resources and insights</li>
                    </ul>
                    
                    <p>Click the button below to accept the invitation and join the team:</p>
                    <p>
                        <a href="{invitation_url}" 
                           style="background-color: #007bff; color: white; padding: 12px 24px; 
                                  text-decoration: none; border-radius: 4px; display: inline-block;">
                            Accept Invitation & Join Team
                        </a>
                    </p>
                    
                    <p>If the button doesn't work, copy and paste the following link into your browser:</p>
                    <p><a href="{invitation_url}">{invitation_url}</a></p>
                    
                    <p><strong>Note:</strong> This invitation will expire in 7 days for security reasons.</p>
                    
                    <p>If you don't want to join this team, you can safely ignore this email.</p>
                    
                    <hr style="margin: 30px 0; border: none; border-top: 1px solid #eee;">
                    <p style="font-size: 12px; color: #666;">
                        Task Backend - Secure Task Management System<br>
                        You received this email because someone invited you to join their team.
                    </p>
                </div>
            </body>
            </html>
            "#,
            name = name,
            team_name = team_name,
            inviter_name = inviter_name,
            invitation_url = invitation_url
        );

        let text_body = format!(
            r#"
You're Invited to Join a Team! - Task Backend

Hello {name},

{inviter_name} has invited you to join the "{team_name}" team on Task Backend.

Team: {team_name}
Invited by: {inviter_name}
Platform: Task Backend

By joining this team, you'll be able to:
- Collaborate on shared tasks and projects
- Access team-specific features and data
- Work together with team members
- Share resources and insights

To accept the invitation and join the team, please click the following link:
{invitation_url}

Note: This invitation will expire in 7 days for security reasons.

If you don't want to join this team, you can safely ignore this email.

---
Task Backend - Secure Task Management System
You received this email because someone invited you to join their team.
            "#,
            name = name,
            team_name = team_name,
            inviter_name = inviter_name,
            invitation_url = invitation_url
        );

        EmailTemplate {
            name: "team_invitation".to_string(),
            subject,
            html_body,
            text_body,
        }
    }

    /// サブスクリプション変更テンプレート
    fn get_subscription_change_template(
        &self,
        name: &str,
        old_tier: &str,
        new_tier: &str,
    ) -> EmailTemplate {
        let subject = format!("Subscription Updated: {} - Task Backend", new_tier);

        let html_body = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="utf-8">
                <title>Subscription Updated</title>
            </head>
            <body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
                <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
                    <h1 style="color: #28a745;">Subscription Updated!</h1>
                    <p>Hello {name},</p>
                    <p>Great news! Your Task Backend subscription has been successfully updated.</p>
                    <div style="background-color: #f8f9fa; padding: 20px; border-radius: 5px; margin: 20px 0;">
                        <h3 style="margin-top: 0; color: #495057;">Subscription Details</h3>
                        <p><strong>Previous Plan:</strong> {old_tier}</p>
                        <p><strong>New Plan:</strong> <span style="color: #28a745; font-weight: bold;">{new_tier}</span></p>
                        <p><strong>Updated On:</strong> {update_time}</p>
                    </div>
                    <h3>What's New with {new_tier}?</h3>
                    <ul>
                        <li>Enhanced features and capabilities</li>
                        <li>Increased limits and quotas</li>
                        <li>Priority support access</li>
                        <li>Advanced task management tools</li>
                    </ul>
                    <p>You can start using your new features immediately. Log in to your account to explore what's new!</p>
                    <div style="text-align: center; margin: 30px 0;">
                        <a href="https://yourapp.com/dashboard" style="background-color: #28a745; color: white; text-decoration: none; padding: 12px 30px; border-radius: 5px; display: inline-block;">Visit Dashboard</a>
                    </div>
                    <p>If you have any questions about your new subscription, please don't hesitate to contact our support team.</p>
                    <p>Thank you for choosing Task Backend!</p>
                </div>
            </body>
            </html>
            "#,
            name = name,
            old_tier = old_tier,
            new_tier = new_tier,
            update_time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        let text_body = format!(
            r#"
Subscription Updated: {new_tier} - Task Backend

Hello {name},

Great news! Your Task Backend subscription has been successfully updated.

Subscription Details:
- Previous Plan: {old_tier}
- New Plan: {new_tier}
- Updated On: {update_time}

What's New with {new_tier}?
- Enhanced features and capabilities
- Increased limits and quotas
- Priority support access
- Advanced task management tools

You can start using your new features immediately. Log in to your account to explore what's new!

Visit your dashboard: https://yourapp.com/dashboard

If you have any questions about your new subscription, please don't hesitate to contact our support team.

Thank you for choosing Task Backend!

---
Task Backend - Secure Task Management System
This is an automated notification about your subscription change.
            "#,
            name = name,
            old_tier = old_tier,
            new_tier = new_tier,
            update_time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        EmailTemplate {
            name: "subscription_change".to_string(),
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

    #[tokio::test]
    async fn test_account_deletion_confirmation_email() {
        let config = EmailConfig {
            development_mode: true,
            ..Default::default()
        };

        let email_service = EmailService::new(config).unwrap();

        // アカウント削除確認メール送信テスト
        let result = email_service
            .send_account_deletion_confirmation_email("test@example.com", "Test User")
            .await;

        assert!(result.is_ok());
    }

    #[test]
    fn test_account_deletion_confirmation_template() {
        let config = EmailConfig {
            development_mode: true,
            ..Default::default()
        };

        let email_service = EmailService::new(config).unwrap();
        let template = email_service.get_account_deletion_confirmation_template("Test User");

        assert_eq!(template.name, "account_deletion_confirmation");
        assert!(template.subject.contains("Account Deletion Confirmation"));
        assert!(template.html_body.contains("Test User"));
        assert!(template.html_body.contains("Permanently Deleted"));
        assert!(template.text_body.contains("Test User"));
        assert!(template.text_body.contains("Permanently Deleted"));
    }
}
