use anyhow::{Context, Result};
use axum::Form;
use axum::extract::State;
use axum::response::{Html, IntoResponse};
use axum_client_ip::ClientIp;
use serde::Deserialize;
use std::ops::AddAssign;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

use crate::db::key_valid;
use crate::state::AppState;
use crate::templates::{AuthTemplate, CompletedAuthTemplate};
use crate::utils::AppError;

#[derive(Deserialize)]
pub struct AuthForm {
    key: Option<String>,
    redirect: Option<String>,
}

pub async fn auth(
    State(state): State<Arc<AppState>>,
    ClientIp(ip): ClientIp,
    Form(auth): Form<AuthForm>,
) -> Result<impl IntoResponse, AppError> {
    if let (Some(key), Some(redirect)) = (&auth.key, &auth.redirect) {
        let uuid = match Uuid::from_str(key) {
            Ok(uuid) => uuid,
            Err(_) => {
                return Ok(Html(
                    AuthTemplate {
                        redirect,
                        message: &"invalid key format".to_string(),
                    }
                    .to_string(),
                ));
            }
        };
        if key_valid(&state.pool, uuid)
            .await
            .context("stupid database error")?
        {
            Ok(Html(CompletedAuthTemplate { key, redirect }.to_string()))
        } else {
            if let std::collections::hash_map::Entry::Vacant(e) =
                state.ip_failed_auths.lock().unwrap().entry(ip)
            {
                e.insert(1);
                Ok(Html(
                    AuthTemplate {
                        redirect,
                        message: &"nice try".to_string(),
                    }
                    .to_string(),
                ))
            } else {
                state
                    .ip_failed_auths
                    .lock()
                    .unwrap()
                    .get_mut(&ip)
                    .unwrap()
                    .add_assign(1);
                if state.ip_failed_auths.lock().unwrap().get(&ip).unwrap() >= &3 {
                    Ok(Html("nop".to_string()))
                } else {
                    Ok(Html(
                        AuthTemplate {
                            redirect,
                            message: &"nice try".to_string(),
                        }
                        .to_string(),
                    ))
                }
            }
        }
    } else if let Some(redirect) = &auth.redirect {
        Ok(Html(
            AuthTemplate {
                redirect,
                message: &"".to_string(),
            }
            .to_string(),
        ))
    } else {
        Ok(Html("i dont even kjnow how you got here".to_string()))
    }
}
