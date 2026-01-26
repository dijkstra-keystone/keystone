use axum::{extract::State, Extension, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::{ApiError, ApiResult},
    models::{Subscription, SubscriptionResponse},
    AppState,
};

pub async fn get_subscription(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> ApiResult<Json<SubscriptionResponse>> {
    let subscription = Subscription::get_for_user(&state.pool, user_id)
        .await?
        .ok_or(ApiError::NotFound("Subscription not found".to_string()))?;

    Ok(Json(SubscriptionResponse {
        tier: subscription.tier,
        status: subscription.status,
        current_period_end: subscription.current_period_end,
    }))
}

#[derive(Debug, Deserialize)]
pub struct CheckoutRequest {
    pub tier: String,
    pub success_url: String,
    pub cancel_url: String,
}

#[derive(Debug, Serialize)]
pub struct CheckoutResponse {
    pub url: String,
}

pub async fn create_checkout(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(req): Json<CheckoutRequest>,
) -> ApiResult<Json<CheckoutResponse>> {
    if state.config.stripe_secret_key.is_empty() {
        return Err(ApiError::BadRequest("Stripe not configured".to_string()));
    }

    #[derive(sqlx::FromRow)]
    struct UserRow {
        wallet_address: String,
        email: Option<String>,
    }

    let user = sqlx::query_as::<_, UserRow>("SELECT wallet_address, email FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&state.pool)
        .await?;

    let price_id = match req.tier.as_str() {
        "dashboard" => std::env::var("STRIPE_DASHBOARD_PRICE_ID")
            .map_err(|_| ApiError::BadRequest("Dashboard price not configured".to_string()))?,
        _ => return Err(ApiError::BadRequest("Invalid tier".to_string())),
    };

    // Create Stripe checkout session via API
    let client = reqwest::Client::new();

    let mut form_data = vec![
        ("mode", "subscription".to_string()),
        ("success_url", req.success_url),
        ("cancel_url", req.cancel_url),
        ("line_items[0][price]", price_id),
        ("line_items[0][quantity]", "1".to_string()),
        ("metadata[user_id]", user_id.to_string()),
        ("metadata[wallet_address]", user.wallet_address),
    ];

    if let Some(email) = user.email {
        form_data.push(("customer_email", email));
    }

    let resp = client
        .post("https://api.stripe.com/v1/checkout/sessions")
        .header("Authorization", format!("Bearer {}", state.config.stripe_secret_key))
        .form(&form_data)
        .send()
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Stripe request failed: {}", e)))?;

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to parse Stripe response: {}", e)))?;

    let url = body["url"]
        .as_str()
        .ok_or(ApiError::Internal(anyhow::anyhow!("No checkout URL in response")))?
        .to_string();

    Ok(Json(CheckoutResponse { url }))
}

#[derive(Debug, Deserialize)]
pub struct PortalRequest {
    pub return_url: String,
}

pub async fn create_portal(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(req): Json<PortalRequest>,
) -> ApiResult<Json<CheckoutResponse>> {
    if state.config.stripe_secret_key.is_empty() {
        return Err(ApiError::BadRequest("Stripe not configured".to_string()));
    }

    let subscription = Subscription::get_for_user(&state.pool, user_id)
        .await?
        .ok_or(ApiError::NotFound("Subscription not found".to_string()))?;

    let customer_id = subscription
        .stripe_customer_id
        .ok_or(ApiError::BadRequest("No Stripe customer".to_string()))?;

    let client = reqwest::Client::new();

    let form_data = [
        ("customer", customer_id.as_str()),
        ("return_url", req.return_url.as_str()),
    ];

    let resp = client
        .post("https://api.stripe.com/v1/billing_portal/sessions")
        .header("Authorization", format!("Bearer {}", state.config.stripe_secret_key))
        .form(&form_data)
        .send()
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Stripe request failed: {}", e)))?;

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to parse Stripe response: {}", e)))?;

    let url = body["url"]
        .as_str()
        .ok_or(ApiError::Internal(anyhow::anyhow!("No portal URL in response")))?
        .to_string();

    Ok(Json(CheckoutResponse { url }))
}
