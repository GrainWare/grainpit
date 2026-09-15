pub mod db;
pub mod routes;
pub mod state;
pub mod templates;
pub mod utils;

use axum::Router;
use axum::extract::State;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum_client_ip::ClientIpSource;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tower_cookies::CookieManagerLayer;
use tracing::info;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::db::get_stats;
use crate::state::AppState;
use crate::templates::IndexTemplate;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::CLOSE))
        .init();

    let shared_state = Arc::new(AppState::new().await.unwrap());

    let app = Router::new()
        .route("/", get(handler))
        .route("/auth", get(routes::auth::auth))
        .route("/account", get(routes::account::account))
        .route("/admin", get(routes::account::admin))
        .route(
            "/api/edit_grainpit_urls",
            post(routes::api::edit_grainpit_urls::edit_grainpit_urls),
        )
        .route("/api/add_user", post(routes::api::add_user::add_user))
        .route(
            "/api/grainpit_urls",
            get(routes::api::grainpit_urls::grainpit_urls),
        )
        .route("/api/submit", post(routes::api::submit::submit))
        .with_state(shared_state)
        .layer(
            ClientIpSource::from_str(
                &std::env::var("IP_SOURCE").unwrap_or("ConnectInfo".to_string()),
            )
            .unwrap()
            .into_extension(),
        )
        .layer(CookieManagerLayer::new());

    let listener = tokio::net::TcpListener::bind(
        std::env::var("GRAINPIT_ADDR").unwrap_or("127.0.0.1:7000".to_string()),
    )
    .await
    .unwrap();
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn handler(State(state): State<Arc<AppState>>) -> Response {
    let stats = get_stats(&state.pool).await.unwrap();
    Html(
        IndexTemplate {
            ips: &stats.unique_ips,
            user_agents: &stats.unique_uas,
            requests: &stats.total_requests,
            grainpit_urls: &stats.grainpit_url_count,
        }
        .to_string(),
    )
    .into_response()
}
