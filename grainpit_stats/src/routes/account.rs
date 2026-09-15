use anyhow::Result;
use axum::extract::State;
use axum::response::{Html, IntoResponse, Redirect, Response};
use std::str::FromStr;
use std::sync::Arc;
use tower_cookies::Cookies;
use uuid::Uuid;

use crate::db::{get_account_from_key, key_valid};
use crate::state::AppState;
use crate::templates::{AccountTemplate, AdminTemplate};
use crate::utils::AppError;

pub async fn account(
    cookies: Cookies,
    State(state): State<Arc<AppState>>,
) -> Result<Response, AppError> {
    let key = cookies.get("key");
    if key.is_none() {
        return Ok(Redirect::to("/auth?redirect=/account").into_response());
    }
    let key = key.unwrap();
    let key = key.value();
    let uuid = match Uuid::from_str(key) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(Redirect::to("/auth?redirect=/account").into_response());
        }
    };
    if !key_valid(&state.pool, uuid).await? {
        return Ok(Redirect::to("/auth?redirect=/account").into_response());
    }

    let account = get_account_from_key(&state.pool, uuid).await?;
    Ok(Html(
        AccountTemplate {
            username: &account.name,
            urls: &account.grainpit_urls.join("\n"),
            admin: if account.name == "admin" {
                &true
            } else {
                &false
            },
        }
        .to_string(),
    )
    .into_response())
}

pub async fn admin(
    cookies: Cookies,
    State(state): State<Arc<AppState>>,
) -> Result<Response, AppError> {
    let key = cookies.get("key");
    if key.is_none() {
        return Ok(Redirect::to("/auth?redirect=/account").into_response());
    }
    let key = key.unwrap();
    let key = key.value();
    let uuid = match Uuid::from_str(key) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(Redirect::to("/auth?redirect=/account").into_response());
        }
    };
    if !key_valid(&state.pool, uuid).await? {
        return Ok(Redirect::to("/auth?redirect=/account").into_response());
    }
    let account = get_account_from_key(&state.pool, uuid).await?;
    if account.name != "admin" {
        return Ok(Redirect::to("/auth?redirect=/account").into_response());
    }
    Ok(Html(AdminTemplate {}.to_string()).into_response())
}
