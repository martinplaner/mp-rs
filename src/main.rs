use askama_axum::{Response, Template};
use axum::extract::{Path, State};
use axum::http::{header, StatusCode, Uri};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use rand::{thread_rng, Rng};
use rust_embed::Embed;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use std::{io, path};
use tokio::signal;

#[derive(Default, Clone)]
struct AppContext {
    dict: Arc<HashMap<char, Vec<String>>>,
    config: Arc<Config>,
}

struct Config {
    delimiter: String,
    default_term: String,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            delimiter: "-".to_owned(),
            default_term: "MP".to_owned(),
        }
    }
}

#[derive(Debug)]
struct AppError {
    message: String,
    status_code: Option<StatusCode>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status_code = self
            .status_code
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status_code, self.message).into_response()
    }
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    query: String,
    result: String,
}

#[derive(Embed)]
#[folder = "assets/"]
struct Assets;

#[tokio::main]
async fn main() {
    let dict = load_dict("words_de.txt").expect("Error loading dictionary");

    let state = AppContext {
        dict: Arc::new(dict),
        config: Arc::new(Config::default()),
    };

    let app_router = Router::new()
        .route("/", get(root_handler))
        .route("/:term", get(term_handler))
        .with_state(state);

    let asset_router = Router::new().route("/_assets/*file", get(asset_handler));

    let router = Router::new().merge(asset_router).merge(app_router);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Can't bind to address");

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn asset_handler(uri: Uri) -> Result<impl IntoResponse, AppError> {
    let path = uri.path().trim_start_matches("/_assets/").to_string();

    match Assets::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(path)
                .first_or_octet_stream()
                .to_string();
            Ok(([(header::CONTENT_TYPE, mime)], content.data))
        }
        None => Err(AppError {
            message: StatusCode::NOT_FOUND.to_string(),
            status_code: Some(StatusCode::NOT_FOUND),
        }),
    }
}

async fn root_handler(state: State<AppContext>) -> Result<impl IntoResponse, AppError> {
    let term = state.config.default_term.clone();
    term_handler(state, Path(term)).await
}

async fn term_handler(
    State(state): State<AppContext>,
    Path(term): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let word = generate_word(&state.dict, &term, &state.config.delimiter).ok_or(AppError {
        message: "Error generating word".to_string(),
        status_code: Some(StatusCode::NOT_FOUND),
    })?;

    Ok(IndexTemplate {
        query: state.config.default_term.clone(),
        result: word,
    })
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
