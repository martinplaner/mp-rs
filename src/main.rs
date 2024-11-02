use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use std::{io, path};
use tokio::signal;

#[derive(Clone)]
struct AppContext {
    dict: Arc<HashMap<char, Vec<String>>>,
    config: Arc<Config>,
}

struct Config {
    delimiter: String,
    default_term: String,
}

#[tokio::main]
async fn main() {
    let dict = load_dict("words_de.txt").expect("Error loading dictionary");

    let state = AppContext {
        dict: Arc::new(dict),
        config: Arc::new(Config {
            delimiter: "-".to_owned(),
            default_term: "mp".to_owned(),
        }),
    };

    let router = Router::new()
        .route("/", get(root_handler))
        .route("/:term", get(term_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Can't bind to address");

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn root_handler(State(state): State<AppContext>) -> impl IntoResponse {
    match generate_word(
        &state.dict,
        &state.config.default_term,
        &state.config.delimiter,
    ) {
        None => (StatusCode::NOT_FOUND, StatusCode::NOT_FOUND.to_string()),
        Some(word) => (StatusCode::OK, word),
    }
}

async fn term_handler(
    State(state): State<AppContext>,
    Path(term): Path<String>,
) -> impl IntoResponse {
    match generate_word(&state.dict, &term, &state.config.delimiter) {
        None => (StatusCode::NOT_FOUND, StatusCode::NOT_FOUND.to_string()),
        Some(word) => (StatusCode::OK, word),
    }
}

fn load_dict(path: &str) -> io::Result<HashMap<char, Vec<String>>> {
    let file = File::open(path::Path::new(path))?;
    let reader = BufReader::new(file);

    let mut dict: HashMap<char, Vec<String>> = HashMap::new();
    for line in reader.lines() {
        let line = line?;

        if line.starts_with('#') {
            continue;
        }

        if let Some(mut first_char) = line.chars().next() {
            first_char = first_char.to_uppercase().next().unwrap_or(first_char);
            dict.entry(first_char).or_default().push(line);
        }
    }

    Ok(dict)
}

fn generate_word(dict: &HashMap<char, Vec<String>>, term: &str, delimiter: &str) -> Option<String> {
    let mut rng = thread_rng();

    term.to_uppercase()
        .chars()
        .map(|c| {
            dict.get(&c).and_then(|words| {
                let i = rng.gen_range(0..words.len());
                words.get(i).map(|s| s.to_owned())
            })
        })
        .collect::<Option<Vec<String>>>()
        .map(|words| words.join(delimiter))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
