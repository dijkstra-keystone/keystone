use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

use crate::{handlers::auth::validate_jwt, AppState};

pub async fn require_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = validate_jwt(&state.config.jwt_secret, token).map_err(|_| StatusCode::UNAUTHORIZED)?;

    request.extensions_mut().insert(claims.user_id);

    Ok(next.run(request).await)
}
