use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use axum::{
    Router,
    extract::{Path, State},
    http::{HeaderMap, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use axum_client_ip::ClientIp;
use axum_client_ip::ClientIpSource;
use axum_extra::{TypedHeader, headers::UserAgent};
use grainpit::{
    markov::Markov,
    stats::{Request, Submission},
};
use tracing::{error, info};
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(feature = "compress")]
use tower_http::compression::CompressionLayer;

#[derive(Debug)]
struct Stats {
    stats_url: String,
    stats_key: String,
    stats_queue: Mutex<Vec<Request>>,
}

#[derive(Debug)]
struct AppState {
    m: Markov,
    stats: Option<Stats>,
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

    let stats_url = std::env::var("GRAINPIT_STATS_URL").ok();
    let stats_key = std::env::var("GRAINPIT_STATS_KEY").ok();

    let stats = if let (Some(stats_url), Some(stats_key)) = (stats_url, stats_key) {
        info!("sending stats to {} with key {}", stats_url, stats_key);
        Some(Stats {
            stats_url,
            stats_key,
            stats_queue: Mutex::new(Vec::new()),
        })
    } else {
        info!("not sending stats");
        None
    };

    let shared_state = Arc::new(AppState {
        m: Markov::new(),
        stats,
    });

    #[cfg(not(feature = "compress"))]
    let app = Router::new()
        .route("/", get(handler))
        .route("/{*wildcard}", get(wildcard_handler))
        .layer(
            ClientIpSource::from_str(
                &std::env::var("GRAINPIT_IP_SOURCE").unwrap_or("ConnectInfo".to_string()),
            )
            .unwrap()
            .into_extension(),
        )
        .with_state(shared_state);

    #[cfg(feature = "compress")]
    let app = Router::new()
        .route("/", get(handler))
        .route("/{*wildcard}", get(wildcard_handler))
        .layer(
            ClientIpSource::from_str(
                &std::env::var("GRAINPIT_IP_SOURCE").unwrap_or("ConnectInfo".to_string()),
            )
            .unwrap()
            .into_extension(),
        )
        .layer(CompressionLayer::new().quality(tower_http::CompressionLevel::Precise(9)))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind(
        std::env::var("GRAINPIT_ADDR").unwrap_or("127.0.0.1:5000".to_string()),
    )
    .await
    .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn wildcard_handler(
    State(state): State<Arc<AppState>>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    ClientIp(ip): ClientIp,
    Path(path): Path<String>,
) -> Response {
    if let Some(stats) = &state.stats {
        stats.stats_queue.lock().unwrap().push(Request {
            time: chrono::Utc::now(),
            url: ("/".to_owned() + &path).to_string(),
            ip: ipnet::IpNet::new(ip, if ip.is_ipv4() { 32 } else { 128 }).unwrap(),
            user_agent: user_agent.to_string(),
        });
        if stats.stats_queue.lock().unwrap().len() >= 1024 {
            let submission = Submission::new(stats.stats_queue.lock().unwrap().to_vec())
                .serialize()
                .unwrap();
            let client = reqwest::Client::new();
            let res = client
                .post(format!("{}api/submit", stats.stats_url))
                .header("Authorization", stats.stats_key.clone())
                .body(submission)
                .send()
                .await
                .map_err(|e| error!("{:?}", e));
            *stats.stats_queue.lock().unwrap() = Vec::new();
        }
    }

    if path.contains(".html") {
        Html(state.m.gen_html()).into_response()
    } else if path.contains(".jpg") || path.contains(".png") {
        let output = grainpit::image::gen_image();
        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, "image/jpeg".parse().unwrap());
        (headers, output).into_response()
    } else {
        let mut a = state.m.config_chain.generate(512);
        a = a.trim_start().to_owned();
        a.into_response()
    }
}

async fn handler(
    State(state): State<Arc<AppState>>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    ClientIp(ip): ClientIp,
) -> Html<String> {
    if let Some(stats) = &state.stats {
        stats.stats_queue.lock().unwrap().push(Request {
            time: chrono::Utc::now(),
            url: "/".to_string(),
            ip: ipnet::IpNet::new(ip, if ip.is_ipv4() { 32 } else { 128 }).unwrap(),
            user_agent: user_agent.to_string(),
        });
        if stats.stats_queue.lock().unwrap().len() >= 1024 {
            let submission = Submission::new(stats.stats_queue.lock().unwrap().to_vec())
                .serialize()
                .unwrap();
            let client = reqwest::Client::new();
            let res = client
                .post(format!("{}api/submit", stats.stats_url))
                .header("Authorization", stats.stats_key.clone())
                .body(submission)
                .send()
                .await
                .map_err(|e| error!("{:?}", e));
            *stats.stats_queue.lock().unwrap() = Vec::new();
        }
    }

    Html(state.m.gen_html())
}
