use anyhow::Result;
use axum::Json;
use axum::body::Bytes;
use axum::extract::{FromRequest, Request, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use grainpit::stats::Submission;
use regex::Regex;
use serde::de::DeserializeOwned;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{Level, span};
use uuid::Uuid;

use crate::db::{edit_user_grainpit_urls, get_account_from_key, insert_submission, key_valid};
use crate::state::AppState;
use crate::utils::AppError;

pub struct Cbor(Submission);

impl<S> FromRequest<S> for Cbor
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(req, state).await.map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("Failed to extract bytes: {}", e),
            )
        })?;
        let value = Submission::deserialize(bytes.to_vec()).map_err(|e| {
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Failed to deserialize CBOR: {}", e),
            )
        })?;
        Ok(Cbor(value))
    }
}

pub async fn submit(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Cbor(submission): Cbor,
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

    insert_submission(&state.pool, submission, account.id).await?;

    Ok((StatusCode::OK, "success").into_response())
}
