use anyhow::Result;
use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use regex::Regex;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

use crate::db::{edit_user_grainpit_urls, key_valid};
use crate::state::AppState;
use crate::utils::AppError;

pub async fn edit_grainpit_urls(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Vec<String>>,
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

    let re = Regex::new(r"^https:\/\/.[^\/]*\/$").unwrap();
    for (idx, url) in payload.iter().enumerate() {
        if !re.is_match(url) {
            return Ok((
                StatusCode::BAD_REQUEST,
                format!("invalid URL at line {} ({})", idx + 1, url),
            )
                .into_response());
        }
    }

    edit_user_grainpit_urls(&state.pool, uuid, payload).await?;
    Ok((StatusCode::OK, "success").into_response())
}
