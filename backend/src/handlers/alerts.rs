use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    error::ApiResult,
    models::{Alert, AlertConfig, UpdateAlertConfigRequest},
    AppState,
};

pub async fn list_alerts(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> ApiResult<Json<Vec<Alert>>> {
    let alerts = Alert::list_for_user(&state.pool, user_id, 50).await?;
    Ok(Json(alerts))
}

pub async fn get_config(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> ApiResult<Json<AlertConfig>> {
    let config = AlertConfig::get_or_create(&state.pool, user_id).await?;
    Ok(Json(config))
}

pub async fn update_config(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(req): Json<UpdateAlertConfigRequest>,
) -> ApiResult<Json<AlertConfig>> {
    AlertConfig::get_or_create(&state.pool, user_id).await?;
    let config = AlertConfig::update(&state.pool, user_id, req).await?;
    Ok(Json(config))
}

pub async fn dismiss_alert(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(alert_id): Path<Uuid>,
) -> ApiResult<Json<()>> {
    Alert::dismiss(&state.pool, alert_id, user_id).await?;
    Ok(Json(()))
}
