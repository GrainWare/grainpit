use anyhow::Result;
use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use regex::Regex;
use serde::Deserialize;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

use crate::db::{create_user, edit_user_grainpit_urls, get_account_from_key, key_valid};
use crate::state::AppState;
use crate::utils::AppError;

#[derive(Deserialize)]
pub struct Payload {
    username: String,
}

pub async fn add_user(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Payload>,
) -> Result<Response, AppError> {
    let key = headers.get("Authorization");
    if key.is_none() {
        return Ok((
            StatusCode::UNAUTHORIZED,
            "please add an Authorization header and try again",
        )
            .into_response());
    }
    let key = key.unwrap().to_str()?;
    let uuid = match Uuid::from_str(key) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok((
                StatusCode::BAD_REQUEST,
                "invalid Authorization header format",
            )
                .into_response());
        }
    };
    if !key_valid(&state.pool, uuid).await? {
        return Ok((StatusCode::UNAUTHORIZED, "invalid Authorization header").into_response());
    }
    let account = get_account_from_key(&state.pool, uuid).await?;
    if account.name != "admin" {
        return Ok((StatusCode::UNAUTHORIZED, "nope").into_response());
    }

    let user_key = create_user(&state.pool, payload.username).await?;

    Ok((StatusCode::OK, user_key.to_string()).into_response())
}
