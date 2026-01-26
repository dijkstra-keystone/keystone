use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub wallet_address: String,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub wallet_address: String,
    pub email: Option<String>,
    pub subscription_tier: String,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub async fn find_by_wallet(
        pool: &sqlx::PgPool,
        wallet: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, wallet_address, email, created_at, updated_at
             FROM users WHERE wallet_address = $1",
        )
        .bind(wallet.to_lowercase())
        .fetch_optional(pool)
        .await
    }

    pub async fn create(pool: &sqlx::PgPool, wallet: &str) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "INSERT INTO users (wallet_address)
             VALUES ($1)
             RETURNING id, wallet_address, email, created_at, updated_at",
        )
        .bind(wallet.to_lowercase())
        .fetch_one(pool)
        .await
    }

    pub async fn update_email(
        pool: &sqlx::PgPool,
        user_id: Uuid,
        email: Option<String>,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "UPDATE users SET email = $2, updated_at = NOW()
             WHERE id = $1
             RETURNING id, wallet_address, email, created_at, updated_at",
        )
        .bind(user_id)
        .bind(email)
        .fetch_one(pool)
        .await
    }
}
