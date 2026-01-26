use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    models::{Subscription, SubscriptionStatus, SubscriptionTier},
    AppState,
};

type HmacSha256 = Hmac<Sha256>;

const TIMESTAMP_TOLERANCE_SECS: u64 = 300;

fn verify_stripe_signature(
    payload: &[u8],
    signature_header: &str,
    secret: &str,
) -> Result<(), StatusCode> {
    let mut timestamp: Option<&str> = None;
    let mut signatures: Vec<&str> = Vec::new();

    for part in signature_header.split(',') {
        let mut kv = part.splitn(2, '=');
        match (kv.next(), kv.next()) {
            (Some("t"), Some(t)) => timestamp = Some(t),
            (Some("v1"), Some(sig)) => signatures.push(sig),
            _ => {}
        }
    }

    let timestamp = timestamp.ok_or(StatusCode::BAD_REQUEST)?;

    if signatures.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let ts: u64 = timestamp.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .as_secs();

    if now.saturating_sub(ts) > TIMESTAMP_TOLERANCE_SECS {
        tracing::warn!("Stripe webhook timestamp too old: {} vs {}", ts, now);
        return Err(StatusCode::BAD_REQUEST);
    }

    let signed_payload = format!("{}.{}", timestamp, String::from_utf8_lossy(payload));

    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    mac.update(signed_payload.as_bytes());
    let expected = mac.finalize().into_bytes();
    let expected_hex = hex::encode(expected);

    for sig in signatures {
        if constant_time_eq(sig.as_bytes(), expected_hex.as_bytes()) {
            return Ok(());
        }
    }

    tracing::warn!("Stripe webhook signature verification failed");
    Err(StatusCode::BAD_REQUEST)
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

pub async fn stripe_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, StatusCode> {
    let signature = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let webhook_secret = std::env::var("STRIPE_WEBHOOK_SECRET").map_err(|_| {
        tracing::error!("STRIPE_WEBHOOK_SECRET not configured");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    verify_stripe_signature(&body, signature, &webhook_secret)?;

    let body_str = std::str::from_utf8(&body).map_err(|_| StatusCode::BAD_REQUEST)?;
    let event: serde_json::Value =
        serde_json::from_str(body_str).map_err(|_| StatusCode::BAD_REQUEST)?;

    let event_type = event["type"].as_str().unwrap_or("");

    match event_type {
        "customer.subscription.created"
        | "customer.subscription.updated"
        | "customer.subscription.deleted" => {
            handle_subscription_event(&state, &event["data"]["object"]).await?;
        }
        "checkout.session.completed" => {
            handle_checkout_completed(&state, &event["data"]["object"]).await?;
        }
        _ => {
            tracing::debug!("Unhandled webhook event: {}", event_type);
        }
    }

    Ok(StatusCode::OK)
}

async fn handle_subscription_event(
    state: &AppState,
    subscription: &serde_json::Value,
) -> Result<(), StatusCode> {
    let customer_id = subscription["customer"]
        .as_str()
        .ok_or(StatusCode::BAD_REQUEST)?;

    let existing = Subscription::find_by_stripe_customer(&state.pool, customer_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let Some(existing) = existing else {
        tracing::warn!("No subscription found for customer {}", customer_id);
        return Ok(());
    };

    let status_str = subscription["status"].as_str().unwrap_or("active");
    let status = match status_str {
        "active" => SubscriptionStatus::Active,
        "canceled" => SubscriptionStatus::Canceled,
        "past_due" => SubscriptionStatus::PastDue,
        "trialing" => SubscriptionStatus::Trialing,
        _ => SubscriptionStatus::Active,
    };

    let tier = if status_str == "canceled" {
        SubscriptionTier::Free
    } else {
        SubscriptionTier::Dashboard
    };

    let period_end = subscription["current_period_end"]
        .as_i64()
        .and_then(|ts| DateTime::<Utc>::from_timestamp(ts, 0));

    let subscription_id = subscription["id"].as_str().unwrap_or("");

    Subscription::update_from_stripe(
        &state.pool,
        existing.user_id,
        customer_id,
        subscription_id,
        tier,
        status,
        period_end,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to update subscription: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(())
}

async fn handle_checkout_completed(
    state: &AppState,
    session: &serde_json::Value,
) -> Result<(), StatusCode> {
    let user_id_str = session["metadata"]["user_id"]
        .as_str()
        .ok_or(StatusCode::BAD_REQUEST)?;

    let user_id = uuid::Uuid::parse_str(user_id_str).map_err(|_| StatusCode::BAD_REQUEST)?;

    let customer_id = session["customer"]
        .as_str()
        .ok_or(StatusCode::BAD_REQUEST)?;

    let subscription_id = session["subscription"]
        .as_str()
        .ok_or(StatusCode::BAD_REQUEST)?;

    Subscription::update_from_stripe(
        &state.pool,
        user_id,
        customer_id,
        subscription_id,
        SubscriptionTier::Dashboard,
        SubscriptionStatus::Active,
        None,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to update subscription after checkout: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(())
}
