use crate::AppState;
use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use std::env;

use super::claims::{UserClaims, UserContext};

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 1. Get Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(header_val) if header_val.starts_with("Bearer ") => {
            &header_val[7..] // Strip "Bearer "
        }
        _ => {
            if !state.config.security.jwt_required {
                return Ok(next.run(request).await);
            }
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // 2. Decode & Validate Token
    let secret = &state.config.security.jwt_secret;
    // Note: In production, cache the DecodingKey
    let key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::default();

    match decode::<UserClaims>(token, &key, &validation) {
        Ok(token_data) => {
            let claims = token_data.claims;
            let context = UserContext {
                user_id: claims.sub.clone(),
                claims,
            };
            // 3. Inject Context
            request.extensions_mut().insert(context);
            Ok(next.run(request).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}
