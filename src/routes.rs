use crate::generator::Generator;
use askama::Template;
use askama_derive_axum::IntoResponse;
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode, Uri};
use axum::routing::get;
use axum::Router;
use axum_core::response::{IntoResponse, Response};
use rust_embed::Embed;
use std::sync::Arc;

const ACCEPT_TEXT_PLAIN: &str = "text/plain";

#[derive(Clone)]
pub struct AppContext {
    pub generator: Arc<dyn Generator + Send + Sync>,
    pub default_term: String,
}

#[derive(Template, IntoResponse)]
#[template(path = "index.html")]
struct IndexTemplate {
    term: String,
    result: String,
}

#[derive(Embed)]
#[folder = "assets/"]
struct Assets;

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

pub fn routes(state: AppContext) -> Router {
    let app_router = Router::new()
        .route("/", get(root_handler))
        .route("/{term}", get(term_handler))
        .with_state(state);

    let asset_router = Router::new().route("/_assets/{*file}", get(asset_handler));

    Router::new().merge(asset_router).merge(app_router)
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
    let term = state.default_term.clone();
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
        term: state.default_term.clone(),
        result: word,
    }
    .into_response())
}
