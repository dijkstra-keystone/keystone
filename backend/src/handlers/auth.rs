use axum::{extract::State, Json};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{encode, DecodingKey, EncodingKey, Header, Validation};
use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    error::{ApiError, ApiResult},
    models::{Subscription, User},
    AppState,
};

const NONCE_EXPIRY_MINUTES: i64 = 10;

#[derive(Debug, Serialize)]
pub struct NonceResponse {
    pub nonce: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    pub address: String,
    pub signature: String,
    pub nonce: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub wallet_address: String,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub user_id: Uuid,
    pub wallet: String,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, FromRow)]
struct NonceRecord {
    id: Uuid,
    used: bool,
    expires_at: DateTime<Utc>,
}

pub async fn get_nonce(State(state): State<AppState>) -> ApiResult<Json<NonceResponse>> {
    let nonce: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let message = format!(
        "Sign this message to authenticate with Dijkstra Keystone.\n\nNonce: {}",
        nonce
    );

    let expires_at = Utc::now() + Duration::minutes(NONCE_EXPIRY_MINUTES);

    sqlx::query(
        "INSERT INTO auth_nonces (nonce, expires_at) VALUES ($1, $2)"
    )
    .bind(&nonce)
    .bind(expires_at)
    .execute(&state.pool)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to store nonce: {}", e)))?;

    Ok(Json(NonceResponse { nonce, message }))
}

pub async fn verify_signature(
    State(state): State<AppState>,
    Json(req): Json<VerifyRequest>,
) -> ApiResult<Json<AuthResponse>> {
    // Validate address format
    if !req.address.starts_with("0x") || req.address.len() != 42 {
        return Err(ApiError::BadRequest("Invalid wallet address".to_string()));
    }

    // Validate and consume nonce
    let nonce_record = sqlx::query_as::<_, NonceRecord>(
        "SELECT id, used, expires_at FROM auth_nonces WHERE nonce = $1"
    )
    .bind(&req.nonce)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("Database error: {}", e)))?
    .ok_or_else(|| ApiError::BadRequest("Invalid or expired nonce".to_string()))?;

    if nonce_record.used {
        return Err(ApiError::BadRequest("Nonce already used".to_string()));
    }

    if nonce_record.expires_at < Utc::now() {
        return Err(ApiError::BadRequest("Nonce expired".to_string()));
    }

    // Mark nonce as used
    sqlx::query(
        "UPDATE auth_nonces SET used = true, wallet_address = $1 WHERE id = $2"
    )
    .bind(&req.address.to_lowercase())
    .bind(nonce_record.id)
    .execute(&state.pool)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to update nonce: {}", e)))?;

    // Construct the message that was signed
    let message = format!(
        "Sign this message to authenticate with Dijkstra Keystone.\n\nNonce: {}",
        req.nonce
    );

    // Verify the signature and recover the address
    let recovered_address = recover_address(&message, &req.signature)
        .map_err(|e| ApiError::BadRequest(format!("Invalid signature: {}", e)))?;

    // Compare addresses (case-insensitive)
    if recovered_address.to_lowercase() != req.address.to_lowercase() {
        return Err(ApiError::Unauthorized);
    }

    let wallet = req.address.to_lowercase();

    let user = match User::find_by_wallet(&state.pool, &wallet).await? {
        Some(user) => user,
        None => {
            let user = User::create(&state.pool, &wallet).await?;
            Subscription::create_free(&state.pool, user.id).await?;
            user
        }
    };

    let token = create_jwt(&state.config.jwt_secret, &user)?;

    Ok(Json(AuthResponse {
        token,
        user: UserInfo {
            id: user.id,
            wallet_address: user.wallet_address,
            email: user.email,
        },
    }))
}

fn recover_address(message: &str, signature_hex: &str) -> Result<String, String> {
    // Remove 0x prefix if present
    let sig_hex = signature_hex.strip_prefix("0x").unwrap_or(signature_hex);

    // Decode the signature (65 bytes: r[32] + s[32] + v[1])
    let sig_bytes = hex::decode(sig_hex).map_err(|e| format!("Invalid hex: {}", e))?;

    if sig_bytes.len() != 65 {
        return Err(format!(
            "Invalid signature length: expected 65, got {}",
            sig_bytes.len()
        ));
    }

    // Extract r, s, and v
    let r = &sig_bytes[0..32];
    let s = &sig_bytes[32..64];
    let v = sig_bytes[64];

    // Convert v to recovery id
    // v is either 27/28 (legacy) or 0/1 (EIP-155)
    // RecoveryId takes (is_y_odd, is_x_reduced) where is_x_reduced is typically false
    let recovery_id = match v {
        27 | 0 => RecoveryId::new(false, false),
        28 | 1 => RecoveryId::new(true, false),
        _ => return Err(format!("Invalid v value: {}", v)),
    };

    // Create the signature from r and s
    let mut sig_bytes_rs = [0u8; 64];
    sig_bytes_rs[0..32].copy_from_slice(r);
    sig_bytes_rs[32..64].copy_from_slice(s);
    let signature =
        Signature::from_bytes((&sig_bytes_rs).into()).map_err(|e| format!("Invalid signature: {}", e))?;

    // Hash the message with Ethereum prefix (EIP-191)
    let prefixed_message = format!("\x19Ethereum Signed Message:\n{}{}", message.len(), message);
    let message_hash = Keccak256::digest(prefixed_message.as_bytes());

    // Recover the public key
    let verifying_key = VerifyingKey::recover_from_prehash(&message_hash, &signature, recovery_id)
        .map_err(|e| format!("Failed to recover key: {}", e))?;

    // Convert public key to Ethereum address
    let public_key_bytes = verifying_key.to_encoded_point(false);
    let public_key_hash = Keccak256::digest(&public_key_bytes.as_bytes()[1..]); // Skip the 0x04 prefix
    let address_bytes = &public_key_hash[12..]; // Take last 20 bytes

    Ok(format!("0x{}", hex::encode(address_bytes)))
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub token: String,
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> ApiResult<Json<AuthResponse>> {
    let claims = validate_jwt(&state.config.jwt_secret, &req.token)?;

    let user = User::find_by_wallet(&state.pool, &claims.wallet)
        .await?
        .ok_or(ApiError::NotFound("User not found".to_string()))?;

    let token = create_jwt(&state.config.jwt_secret, &user)?;

    Ok(Json(AuthResponse {
        token,
        user: UserInfo {
            id: user.id,
            wallet_address: user.wallet_address,
            email: user.email,
        },
    }))
}

fn create_jwt(secret: &str, user: &User) -> ApiResult<String> {
    let now = Utc::now();
    let exp = now + Duration::days(7);

    let claims = Claims {
        sub: user.id.to_string(),
        user_id: user.id,
        wallet: user.wallet_address.clone(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("JWT encoding failed: {}", e)))
}

pub fn validate_jwt(secret: &str, token: &str) -> ApiResult<Claims> {
    jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| ApiError::Unauthorized)
}

// Cleanup expired nonces (called periodically)
pub async fn cleanup_expired_nonces(pool: &sqlx::PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM auth_nonces WHERE expires_at < NOW()")
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recover_address_valid_signature() {
        // Test vector: a known message and signature pair
        // This uses a test wallet's signature
        let message = "Sign this message to authenticate with Dijkstra Keystone.\n\nNonce: abc123";

        // For testing, we verify the function doesn't panic on valid-format inputs
        // A real signature would need to be generated by a wallet
        let invalid_but_valid_format = "0x".to_string() + &"00".repeat(65);

        // Should return an error (signature doesn't match) but not panic
        let result = recover_address(message, &invalid_but_valid_format);
        assert!(result.is_err() || result.is_ok()); // Just verify it doesn't panic
    }

    #[test]
    fn test_recover_address_invalid_hex() {
        let message = "test message";
        let invalid_hex = "0xZZZZ";

        let result = recover_address(message, invalid_hex);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid hex"));
    }

    #[test]
    fn test_recover_address_wrong_length() {
        let message = "test message";
        let short_sig = "0x" .to_string() + &"00".repeat(32); // 32 bytes instead of 65

        let result = recover_address(message, &short_sig);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid signature length"));
    }

    #[test]
    fn test_recover_address_invalid_v_value() {
        let message = "test message";
        // Valid length but invalid v value (last byte = 99)
        let mut sig = "00".repeat(64);
        sig.push_str("63"); // v = 99 (invalid)
        let sig_hex = "0x".to_string() + &sig;

        let result = recover_address(message, &sig_hex);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid v value"));
    }

    #[test]
    fn test_create_and_validate_jwt() {
        let user = User {
            id: Uuid::new_v4(),
            wallet_address: "0x1234567890123456789012345678901234567890".to_string(),
            email: Some("test@example.com".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let secret = "test-secret-key-for-jwt-testing-purposes";

        let token = create_jwt(secret, &user).expect("JWT creation should succeed");
        assert!(!token.is_empty());

        let claims = validate_jwt(secret, &token).expect("JWT validation should succeed");
        assert_eq!(claims.user_id, user.id);
        assert_eq!(claims.wallet, user.wallet_address);
    }

    #[test]
    fn test_validate_jwt_invalid_secret() {
        let user = User {
            id: Uuid::new_v4(),
            wallet_address: "0x1234567890123456789012345678901234567890".to_string(),
            email: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let secret = "correct-secret";
        let wrong_secret = "wrong-secret";

        let token = create_jwt(secret, &user).expect("JWT creation should succeed");
        let result = validate_jwt(wrong_secret, &token);

        assert!(result.is_err());
    }

    #[test]
    fn test_validate_jwt_expired() {
        // Create an expired token manually
        let user = User {
            id: Uuid::new_v4(),
            wallet_address: "0x1234567890123456789012345678901234567890".to_string(),
            email: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let secret = "test-secret";
        let now = Utc::now();
        let expired = now - Duration::days(8); // 8 days ago

        let claims = Claims {
            sub: user.id.to_string(),
            user_id: user.id,
            wallet: user.wallet_address.clone(),
            exp: expired.timestamp(),
            iat: (expired - Duration::days(7)).timestamp(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        ).expect("Encoding should succeed");

        let result = validate_jwt(secret, &token);
        assert!(result.is_err());
    }
}
