pub mod markov;

use std::sync::Arc;

use crate::markov::Chain;
use axum::{
    Router,
    extract::{Path, State},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use rand::seq::IteratorRandom;
use regex::regex;
use tracing::info_span;
use tracing_subscriber::fmt::format::FmtSpan;

#[derive(Debug)]
struct AppState {
    markov1: Chain,
    markov2: Chain,
    markov3: Chain,
    markov4: Chain,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let mut chain1 = Chain::default();
    let mut chain2 = Chain::default();
    let mut chain3 = Chain::default();
    let mut chain4 = Chain::default();

    info_span!("train_markov1").in_scope(|| {
        let mut lines = include_str!("markov1.txt").lines();
        while let Some(_) = lines.next() {
            let mut buffer = String::new();
            for _ in 0..512 {
                if let Some(next_line) = lines.next() {
                    buffer = buffer + "\n" + next_line;
                }
            }
            chain1.train(&buffer.to_owned());
        }
    });

    info_span!("train_markov2").in_scope(|| {
        for line in include_str!("markov2.txt").lines() {
            chain2.train(line);
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
            chain3.train(&buffer.to_owned());
        }
    });

    info_span!("train_markov4").in_scope(|| {
        for line in include_str!("markov4.txt").lines() {
            chain4.train(line);
        }
    });

    let shared_state = Arc::new(AppState {
        markov1: chain1,
        markov2: chain2,
        markov3: chain3,
        markov4: chain4,
    });
    let app = Router::new()
        .route("/", get(handler))
        .route("/{*wildcard}", get(wildcard_handler))
        .with_state(shared_state);

    // run it
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
) -> Response {
    if path.contains(".html") {
        handler(State(state)).await.into_response()
    } else {
        info_span!("wildcard_handler").in_scope(|| {
            let mut a = state.markov3.generate(512);
            a = a.trim_start().to_owned();
            a.into_response()
        })
    }
}

#[tracing::instrument(skip_all)]
async fn handler(State(state): State<Arc<AppState>>) -> Html<String> {
    let mut fakeurls = String::new();
    for _ in 0..16 {
        fakeurls.push_str(&format!(
            "\n<a href='{}'>{}</a><br>",
            random_link(&state.markov4),
            state.markov2.generate(16)
        ));
    }

    let href_regex = regex!(r#"href="(.*?)""#);
    let mut generated = state.markov1.generate(4096);
    for i in href_regex.find_iter(&generated.clone()) {
        generated = generated.replace(i.as_str(), &random_link(&state.markov4));
    }

    Html(format!(
        "<h1>This is my website and it is Amazing!!</h1>\n{}\n<p>{}</p>",
        fakeurls, generated
    ))
}

fn random_link(markov4: &Chain) -> String {
    let start_path = if std::env::var("GRAINPIT_EXTRAURLS").is_ok() {
        if rand::random_bool(
            std::env::var("GRAINPIT_EXTRAURLS_CHANCE")
                .unwrap_or("5".to_owned())
                .parse::<u8>()
                .unwrap() as f64
                / 100.0,
        ) {
            let rng = &mut rand::rng();
            std::env::var("GRAINPIT_EXTRAURLS")
                .unwrap()
                .split(",")
                .choose(rng)
                .unwrap()
                .to_owned()
        } else {
            "/".to_string()
        }
    } else {
        "/".to_string()
    };

    format!(
        "{}{}/{}/{}.html",
        start_path,
        markov4.generate(4),
        markov4.generate(4),
        markov4.generate(24)
    )
}
