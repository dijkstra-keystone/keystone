use anyhow::{Context, Result};

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub stripe_secret_key: String,
    pub stripe_webhook_secret: String,
    pub smtp_host: Option<String>,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub from_email: String,
    pub allowed_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3001".to_string())
                .parse()
                .context("PORT must be a number")?,
            database_url: std::env::var("DATABASE_URL")
                .context("DATABASE_URL must be set")?,
            jwt_secret: std::env::var("JWT_SECRET")
                .context("JWT_SECRET must be set")?,
            stripe_secret_key: std::env::var("STRIPE_SECRET_KEY")
                .unwrap_or_default(),
            stripe_webhook_secret: std::env::var("STRIPE_WEBHOOK_SECRET")
                .unwrap_or_default(),
            smtp_host: std::env::var("SMTP_HOST").ok(),
            smtp_username: std::env::var("SMTP_USERNAME").ok(),
            smtp_password: std::env::var("SMTP_PASSWORD").ok(),
            from_email: std::env::var("FROM_EMAIL")
                .unwrap_or_else(|_| "alerts@dijkstrakeystone.com".to_string()),
            allowed_origins: std::env::var("ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "https://dijkstrakeystone.com,https://www.dijkstrakeystone.com".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        })
    }
}
