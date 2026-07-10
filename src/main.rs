pub mod markov;

use std::sync::Arc;

use crate::markov::Chain;
use axum::{
    Router,
    extract::{Path, State},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use rand::distr::{Alphanumeric, SampleString};
use tracing::info_span;
use tracing_subscriber::fmt::format::FmtSpan;

#[derive(Debug)]
struct AppState {
    markov1: Chain<String>,
    markov2: Chain<String>,
    markov3: Chain<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let mut chain1 = Chain::new();
    let mut chain2 = Chain::new();
    let mut chain3 = Chain::new();

    info_span!("train_markov1").in_scope(|| {
        let mut lines = include_str!("markov1.txt").lines();
        while let Some(_) = lines.next() {
            let mut buffer = String::new();
            for _ in 0..512 {
                if let Some(next_line) = lines.next() {
                    buffer = buffer + "\n" + next_line;
                }
            }
            chain1.feed_str(&buffer.to_owned());
        }
    });

    info_span!("train_markov2").in_scope(|| {
        for line in include_str!("markov2.txt").lines() {
            chain2.feed_str(line);
        }
    });

    info_span!("train_markov3").in_scope(|| {
        let mut lines = include_str!("markov3.txt").lines();
        while let Some(_) = lines.next() {
            let mut buffer = String::new();
            for _ in 0..50 {
                if let Some(next_line) = lines.next() {
                    buffer = buffer + "\n" + next_line;
                }
            }
            chain3.feed_str(&buffer.to_owned());
        }
    });

    let shared_state = Arc::new(AppState {
        markov1: chain1,
        markov2: chain2,
        markov3: chain3,
    });
    let app = Router::new()
        .route("/", get(handler))
        .route("/{*wildcard}", get(wildcard_handler))
        .with_state(shared_state);

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:5000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn wildcard_handler(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Response {
    if path.len() == 69 {
        handler(State(state)).await.into_response()
    } else {
        info_span!("wildcard_handler").in_scope(|| {
            let mut a = state.markov3.generate_str();
            a = a.trim_start().to_owned();
            a.into_response()
        })
    }
}

#[tracing::instrument(skip_all)]
async fn handler(State(state): State<Arc<AppState>>) -> Html<String> {
    let mut fakeurls = String::new();
    for _ in 0..8 {
        fakeurls.push_str(&format!(
            "\n<a href='/{}.html'>{}</a><br>",
            Alphanumeric.sample_string(&mut rand::rng(), 64),
            state.markov2.generate_str()
        ));
    }

    Html(format!(
        "<h1>This is my website and it is Amazing!!</h1>\n{}\n<p>{}</p>",
        fakeurls,
        state.markov1.generate_str()
    ))
}
