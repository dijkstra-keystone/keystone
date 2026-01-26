use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};

use crate::{handlers, middleware::require_auth, AppState};

pub fn api_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/auth", auth_routes())
        .nest("/users", protected_user_routes(state.clone()))
        .nest("/alerts", protected_alert_routes(state.clone()))
        .nest("/subscriptions", protected_subscription_routes(state.clone()))
        .nest("/webhooks", webhook_routes())
}

fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/nonce", get(handlers::auth::get_nonce))
        .route("/verify", post(handlers::auth::verify_signature))
        .route("/refresh", post(handlers::auth::refresh_token))
}

fn protected_user_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/me", get(handlers::users::get_current_user))
        .route("/me", post(handlers::users::update_user))
        .layer(axum_middleware::from_fn_with_state(state, require_auth))
}

fn protected_alert_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::alerts::list_alerts))
        .route("/config", get(handlers::alerts::get_config))
        .route("/config", post(handlers::alerts::update_config))
        .route("/:id/dismiss", post(handlers::alerts::dismiss_alert))
        .layer(axum_middleware::from_fn_with_state(state, require_auth))
}

fn protected_subscription_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::subscriptions::get_subscription))
        .route("/checkout", post(handlers::subscriptions::create_checkout))
        .route("/portal", post(handlers::subscriptions::create_portal))
        .layer(axum_middleware::from_fn_with_state(state, require_auth))
}

fn webhook_routes() -> Router<AppState> {
    Router::new()
        .route("/stripe", post(handlers::webhooks::stripe_webhook))
}
