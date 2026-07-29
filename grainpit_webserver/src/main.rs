use std::sync::Arc;

use axum::{
    Router,
    extract::{Path, State},
    http::{HeaderMap, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use grainpit::markov::Markov;
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(feature = "compress")]
use tower_http::compression::CompressionLayer;
#[derive(Debug)]
struct AppState {
    m: Markov,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,grainpit=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::CLOSE))
        .init();

    let shared_state = Arc::new(AppState { m: Markov::new() });

    #[cfg(not(feature = "compress"))]
    let app = Router::new()
        .route("/", get(handler))
        .route("/{*wildcard}", get(wildcard_handler))
        .with_state(shared_state);

    #[cfg(feature = "compress")]
    let app = Router::new()
        .route("/", get(handler))
        .route("/{*wildcard}", get(wildcard_handler))
        .layer(CompressionLayer::new())
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind(
        std::env::var("GRAINPIT_ADDR").unwrap_or("127.0.0.1:5000".to_string()),
    )
    .await
    .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn wildcard_handler(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    headers: HeaderMap,
) -> Response {
    if let Some(header) = headers.get("Accept-Encoding")
        && header.to_str().unwrap_or("").contains("br")
    {
        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, "text/plain".parse().unwrap());
        headers.insert(header::CONTENT_ENCODING, "br".parse().unwrap());
        return (headers, include_bytes!("./g.txt.br")).into_response();
    }

    if path.contains(".html") {
        handler(State(state)).await.into_response()
    } else {
        let mut a = state.m.config_chain.generate(512);
        a = a.trim_start().to_owned();
        a.into_response()
    }
}

async fn handler(State(state): State<Arc<AppState>>) -> Html<String> {
    Html(state.m.gen_html())
}
