pub mod markov;

use std::sync::Arc;

use crate::markov::Chain;
use axum::{Router, extract::State, response::Html, routing::get};
use rand::{
    RngExt,
    distr::{Alphanumeric, SampleString},
    rng,
};

struct AppState {
    markov1: Chain<String>,
    markov2: Chain<String>,
}

#[tokio::main]
async fn main() {
    let mut chain1 = Chain::new();
    let mut chain2 = Chain::new();

    let mut lines = include_str!("markov1.txt").lines();
    while let Some(line) = lines.next() {
        let mut buffer = String::new();
        for _ in 0..512 {
            if let Some(next_line) = lines.next() {
                buffer = buffer + "<br>" + next_line;
            }
        }
        chain1.feed_str(&buffer.to_owned());
    }

    for line in include_str!("markov2.txt").lines() {
        chain2.feed_str(line);
    }

    let shared_state = Arc::new(AppState {
        markov1: chain1,
        markov2: chain2,
    });
    let app = Router::new()
        .route("/", get(handler))
        .route("/{*wildcard}", get(handler))
        .with_state(shared_state);

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:5000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler(State(state): State<Arc<AppState>>) -> Html<String> {
    let mut fakeurls = String::new();
    for _ in 0..8 {
        fakeurls.push_str(&format!(
            "\n<a href='/{}'>{}</a><br>",
            Alphanumeric.sample_string(&mut rand::rng(), 512),
            state.markov2.generate_str()
        ));
    }

    Html(format!(
        "<h1>This is my website and it is Amazing!!</h1>\n{}\n<p>{}</p>",
        fakeurls,
        state.markov1.generate_str()
    ))
}
