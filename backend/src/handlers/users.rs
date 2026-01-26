use axum::{extract::State, Extension, Json};
use uuid::Uuid;

use crate::{
    error::ApiResult,
    models::{Subscription, SubscriptionTier, UpdateUserRequest, User, UserResponse},
    AppState,
};

pub async fn get_current_user(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> ApiResult<Json<UserResponse>> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, wallet_address, email, created_at, updated_at FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_one(&state.pool)
    .await?;

    let subscription = Subscription::get_for_user(&state.pool, user_id)
        .await?
        .map(|s| s.tier)
        .unwrap_or(SubscriptionTier::Free);

    Ok(Json(UserResponse {
        id: user.id,
        wallet_address: user.wallet_address,
        email: user.email,
        subscription_tier: format!("{:?}", subscription).to_lowercase(),
        created_at: user.created_at,
    }))
}

pub async fn update_user(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> ApiResult<Json<UserResponse>> {
    let user = User::update_email(&state.pool, user_id, req.email).await?;

    let subscription = Subscription::get_for_user(&state.pool, user_id)
        .await?
        .map(|s| s.tier)
        .unwrap_or(SubscriptionTier::Free);

    Ok(Json(UserResponse {
        id: user.id,
        wallet_address: user.wallet_address,
        email: user.email,
        subscription_tier: format!("{:?}", subscription).to_lowercase(),
        created_at: user.created_at,
    }))
}
