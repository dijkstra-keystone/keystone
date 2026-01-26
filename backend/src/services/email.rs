use anyhow::Result;
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};

use crate::config::Config;

pub struct EmailService {
    mailer: Option<AsyncSmtpTransport<Tokio1Executor>>,
    from_email: String,
}

impl EmailService {
    pub fn new(config: &Config) -> Self {
        let mailer = match (&config.smtp_host, &config.smtp_username, &config.smtp_password) {
            (Some(host), Some(username), Some(password)) => {
                let creds = Credentials::new(username.clone(), password.clone());
                AsyncSmtpTransport::<Tokio1Executor>::relay(host)
                    .ok()
                    .map(|builder| builder.credentials(creds).build())
            }
            _ => None,
        };

        Self {
            mailer,
            from_email: config.from_email.clone(),
        }
    }

    pub async fn send_alert(&self, to: &str, subject: &str, body: &str) -> Result<()> {
        let Some(mailer) = &self.mailer else {
            tracing::warn!("Email not configured, skipping send to {}", to);
            return Ok(());
        };

        let email = Message::builder()
            .from(self.from_email.parse()?)
            .to(to.parse()?)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body.to_string())?;

        mailer.send(email).await?;
        tracing::info!("Alert email sent to {}", to);

        Ok(())
    }
}
