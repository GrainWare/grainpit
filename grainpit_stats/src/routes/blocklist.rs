use std::sync::Arc;

use crate::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub async fn blocklist(State(state): State<Arc<AppState>>) -> Response {
    let blocklist: Vec<String> = sqlx::query_scalar(
        "select host(ip) from (select ip, count(*) from request group by ip) where count > 25;",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap();
    (StatusCode::OK, blocklist.join("\n")).into_response()
}
