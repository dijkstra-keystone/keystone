use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "subscription_tier", rename_all = "lowercase")]
pub enum SubscriptionTier {
    Free,
    Dashboard,
    Protocol,
}

impl Default for SubscriptionTier {
    fn default() -> Self {
        Self::Free
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "subscription_status", rename_all = "lowercase")]
pub enum SubscriptionStatus {
    Active,
    Canceled,
    PastDue,
    Trialing,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Subscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tier: SubscriptionTier,
    pub status: SubscriptionStatus,
    pub stripe_customer_id: Option<String>,
    pub stripe_subscription_id: Option<String>,
    pub current_period_end: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionResponse {
    pub tier: SubscriptionTier,
    pub status: SubscriptionStatus,
    pub current_period_end: Option<DateTime<Utc>>,
}

impl Subscription {
    pub async fn get_for_user(pool: &sqlx::PgPool, user_id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Subscription>(
            "SELECT id, user_id, tier, status, stripe_customer_id, stripe_subscription_id,
                    current_period_end, created_at, updated_at
             FROM subscriptions WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn create_free(pool: &sqlx::PgPool, user_id: Uuid) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Subscription>(
            "INSERT INTO subscriptions (user_id, tier, status)
             VALUES ($1, 'free', 'active')
             RETURNING id, user_id, tier, status, stripe_customer_id, stripe_subscription_id,
                       current_period_end, created_at, updated_at",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
    }

    pub async fn update_from_stripe(
        pool: &sqlx::PgPool,
        user_id: Uuid,
        stripe_customer_id: &str,
        stripe_subscription_id: &str,
        tier: SubscriptionTier,
        status: SubscriptionStatus,
        period_end: Option<DateTime<Utc>>,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Subscription>(
            "UPDATE subscriptions SET
               stripe_customer_id = $2,
               stripe_subscription_id = $3,
               tier = $4,
               status = $5,
               current_period_end = $6,
               updated_at = NOW()
             WHERE user_id = $1
             RETURNING id, user_id, tier, status, stripe_customer_id, stripe_subscription_id,
                       current_period_end, created_at, updated_at",
        )
        .bind(user_id)
        .bind(stripe_customer_id)
        .bind(stripe_subscription_id)
        .bind(tier)
        .bind(status)
        .bind(period_end)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_stripe_customer(
        pool: &sqlx::PgPool,
        customer_id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Subscription>(
            "SELECT id, user_id, tier, status, stripe_customer_id, stripe_subscription_id,
                    current_period_end, created_at, updated_at
             FROM subscriptions WHERE stripe_customer_id = $1",
        )
        .bind(customer_id)
        .fetch_optional(pool)
        .await
    }
}
