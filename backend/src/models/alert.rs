use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "alert_severity", rename_all = "lowercase")]
pub enum AlertSeverity {
    Info,
    Warning,
    Danger,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Alert {
    pub id: Uuid,
    pub user_id: Uuid,
    pub severity: AlertSeverity,
    pub alert_type: String,
    pub message: String,
    pub wallet_address: String,
    pub dismissed: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AlertConfig {
    pub id: Uuid,
    pub user_id: Uuid,
    pub enabled: bool,
    pub health_threshold: f64,
    pub webhook_url: Option<String>,
    pub email_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAlertConfigRequest {
    pub enabled: Option<bool>,
    pub health_threshold: Option<f64>,
    pub webhook_url: Option<String>,
    pub email_enabled: Option<bool>,
}

impl Alert {
    pub async fn list_for_user(
        pool: &sqlx::PgPool,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Alert>(
            "SELECT id, user_id, severity, alert_type, message, wallet_address, dismissed, created_at
             FROM alerts
             WHERE user_id = $1 AND dismissed = false
             ORDER BY created_at DESC
             LIMIT $2",
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(pool)
        .await
    }

    pub async fn create(
        pool: &sqlx::PgPool,
        user_id: Uuid,
        severity: AlertSeverity,
        alert_type: &str,
        message: &str,
        wallet_address: &str,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Alert>(
            "INSERT INTO alerts (user_id, severity, alert_type, message, wallet_address)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, user_id, severity, alert_type, message, wallet_address, dismissed, created_at",
        )
        .bind(user_id)
        .bind(severity)
        .bind(alert_type)
        .bind(message)
        .bind(wallet_address)
        .fetch_one(pool)
        .await
    }

    pub async fn dismiss(pool: &sqlx::PgPool, alert_id: Uuid, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE alerts SET dismissed = true WHERE id = $1 AND user_id = $2")
            .bind(alert_id)
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(())
    }
}

impl AlertConfig {
    pub async fn get_or_create(pool: &sqlx::PgPool, user_id: Uuid) -> Result<Self, sqlx::Error> {
        let existing = sqlx::query_as::<_, AlertConfig>(
            "SELECT id, user_id, enabled, health_threshold, webhook_url,
                    email_enabled, created_at, updated_at
             FROM alert_configs WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        if let Some(config) = existing {
            return Ok(config);
        }

        sqlx::query_as::<_, AlertConfig>(
            "INSERT INTO alert_configs (user_id)
             VALUES ($1)
             RETURNING id, user_id, enabled, health_threshold, webhook_url,
                       email_enabled, created_at, updated_at",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
    }

    pub async fn update(
        pool: &sqlx::PgPool,
        user_id: Uuid,
        req: UpdateAlertConfigRequest,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, AlertConfig>(
            "UPDATE alert_configs SET
               enabled = COALESCE($2, enabled),
               health_threshold = COALESCE($3, health_threshold),
               webhook_url = COALESCE($4, webhook_url),
               email_enabled = COALESCE($5, email_enabled),
               updated_at = NOW()
             WHERE user_id = $1
             RETURNING id, user_id, enabled, health_threshold, webhook_url,
                       email_enabled, created_at, updated_at",
        )
        .bind(user_id)
        .bind(req.enabled)
        .bind(req.health_threshold)
        .bind(req.webhook_url)
        .bind(req.email_enabled)
        .fetch_one(pool)
        .await
    }
}
