mod generator;

use crate::generator::{CompoundGenerator, Generator};
use askama_axum::{Response, Template};
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode, Uri};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use clap::Parser;
use rust_embed::Embed;
use std::fs::File;
use std::io::BufReader;
use std::path;
use std::process::exit;
use std::sync::Arc;
use tokio::signal;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to word list (one word per line)
    #[arg(short, long, default_value = "words.txt")]
    file: String,

    /// TCP address for the server to listen on, in the form 'host:port'
    #[arg(short, long, default_value = "0.0.0.0:8080")]
    listen: String,

    /// Default fallback query term, if not provided
    #[arg(short, long, default_value = "MP")]
    default_query: String,

    /// Run generation once with the given query and print result, then quit
    #[arg(short, long)]
    once: Option<String>,
}

#[derive(Clone)]
struct AppContext {
    generator: Arc<dyn Generator + Send + Sync>,
    config: Arc<Config>,
}

struct Config {
    default_term: String,
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
    term: String,
    result: String,
}

#[derive(Embed)]
#[folder = "assets/"]
struct Assets;

const DELIMITER: &str = "-";
const ACCEPT_TEXT_PLAIN: &str = "text/plain";

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let file = File::open(path::Path::new(&cli.file)).expect("Error loading file");
    let reader = BufReader::new(file);

    let generator =
        CompoundGenerator::new(reader, DELIMITER).expect("Error initializing generator");

    if let Some(term) = cli.once {
        let word = generator.generate(&term).expect("Error generating word");
        println!("{}", word);
        exit(0);
    };

    let state = AppContext {
        generator: Arc::new(generator),
        config: Arc::new(Config {
            default_term: cli.default_query,
        }),
    };

    let app_router = Router::new()
        .route("/", get(root_handler))
        .route("/:term", get(term_handler))
        .with_state(state);

    let asset_router = Router::new().route("/_assets/*file", get(asset_handler));

    let router = Router::new().merge(asset_router).merge(app_router);

    let listener = tokio::net::TcpListener::bind(cli.listen)
        .await
        .expect("Can't bind to address");

    println!("Listening on {}", listener.local_addr().unwrap());

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

async fn root_handler(
    state: State<AppContext>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let term = state.config.default_term.clone();
    term_handler(state, Path(term), headers).await
}

async fn term_handler(
    State(state): State<AppContext>,
    Path(term): Path<String>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let word = state.generator.generate(&term).ok_or(AppError {
        message: "Error generating word".to_string(),
        status_code: Some(StatusCode::NOT_FOUND),
    })?;

    if headers
        .get(header::ACCEPT)
        .and_then(|t| t.to_str().ok())
        .map_or(false, |t| t.contains(ACCEPT_TEXT_PLAIN))
    {
        return Ok((StatusCode::OK, word).into_response());
    }

    Ok(IndexTemplate {
        term: state.config.default_term.clone(),
        result: word,
    }
    .into_response())
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
