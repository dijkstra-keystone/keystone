use std::time::Duration;
use tokio::time::interval;

use crate::{
    config::Config,
    models::{Alert, AlertSeverity},
    services::{fetch_portfolio, EmailService},
};

pub async fn run_alert_worker(pool: sqlx::PgPool, config: Config) {
    let email_service = EmailService::new(&config);
    let mut ticker = interval(Duration::from_secs(60));

    loop {
        ticker.tick().await;

        if let Err(e) = check_all_alerts(&pool, &email_service).await {
            tracing::error!("Alert worker error: {}", e);
        }
    }
}

async fn check_all_alerts(pool: &sqlx::PgPool, email_service: &EmailService) -> anyhow::Result<()> {
    let configs: Vec<AlertConfigWithUser> = sqlx::query_as::<_, AlertConfigWithUser>(
        "SELECT ac.id, ac.user_id, ac.enabled, ac.health_threshold,
                ac.webhook_url, ac.email_enabled, u.wallet_address, u.email as user_email
         FROM alert_configs ac
         JOIN users u ON u.id = ac.user_id
         WHERE ac.enabled = true",
    )
    .fetch_all(pool)
    .await?;

    for config in configs {
        if let Err(e) = check_user_alerts(pool, email_service, &config).await {
            tracing::error!("Error checking alerts for user {}: {}", config.user_id, e);
        }
    }

    Ok(())
}

#[derive(Debug, sqlx::FromRow)]
struct AlertConfigWithUser {
    id: uuid::Uuid,
    user_id: uuid::Uuid,
    enabled: bool,
    health_threshold: f64,
    webhook_url: Option<String>,
    email_enabled: bool,
    wallet_address: String,
    user_email: Option<String>,
}

async fn check_user_alerts(
    pool: &sqlx::PgPool,
    email_service: &EmailService,
    config: &AlertConfigWithUser,
) -> anyhow::Result<()> {
    let portfolio = fetch_portfolio(&config.wallet_address).await?;

    if portfolio.health_factor < 1.0 {
        create_and_notify_alert(
            pool,
            email_service,
            config,
            AlertSeverity::Critical,
            "health_factor_critical",
            &format!(
                "CRITICAL: Health factor is {:.2}. Liquidation imminent!",
                portfolio.health_factor
            ),
        )
        .await?;
    } else if portfolio.health_factor < config.health_threshold {
        create_and_notify_alert(
            pool,
            email_service,
            config,
            AlertSeverity::Warning,
            "health_factor_low",
            &format!(
                "Health factor dropped to {:.2} (threshold: {:.2})",
                portfolio.health_factor, config.health_threshold
            ),
        )
        .await?;
    }

    if portfolio.liquidation_distance < 10.0 {
        create_and_notify_alert(
            pool,
            email_service,
            config,
            AlertSeverity::Danger,
            "liquidation_near",
            &format!(
                "Only {:.1}% away from liquidation!",
                portfolio.liquidation_distance
            ),
        )
        .await?;
    }

    Ok(())
}

async fn create_and_notify_alert(
    pool: &sqlx::PgPool,
    email_service: &EmailService,
    config: &AlertConfigWithUser,
    severity: AlertSeverity,
    alert_type: &str,
    message: &str,
) -> anyhow::Result<()> {
    let recent: Option<i64> = sqlx::query_scalar(
        "SELECT COUNT(*) FROM alerts
         WHERE user_id = $1 AND alert_type = $2
         AND created_at > NOW() - INTERVAL '5 minutes'",
    )
    .bind(config.user_id)
    .bind(alert_type)
    .fetch_one(pool)
    .await?;

    if recent.unwrap_or(0) > 0 {
        return Ok(());
    }

    let alert = Alert::create(
        pool,
        config.user_id,
        severity,
        alert_type,
        message,
        &config.wallet_address,
    )
    .await?;

    if let Some(webhook_url) = &config.webhook_url {
        send_webhook(webhook_url, &alert).await;
    }

    if config.email_enabled {
        if let Some(email) = &config.user_email {
            let subject = format!("Keystone Alert: {}", alert_type.replace('_', " "));
            if let Err(e) = email_service.send_alert(email, &subject, message).await {
                tracing::warn!("Failed to send alert email: {}", e);
            }
        }
    }

    Ok(())
}

async fn send_webhook(url: &str, alert: &Alert) {
    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "type": alert.alert_type,
        "severity": format!("{:?}", alert.severity).to_lowercase(),
        "message": alert.message,
        "wallet": alert.wallet_address,
        "timestamp": alert.created_at.to_rfc3339(),
    });

    if let Err(e) = client.post(url).json(&payload).send().await {
        tracing::warn!("Webhook delivery failed to {}: {}", url, e);
    }
}
