use axum::{
    Router,
    extract::{Path, Query, Request, State},
    http::{HeaderMap, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use clap::Parser;
use reqwest::Client;
use rss::{Channel, Item};
use serde::Deserialize;
use std::sync::Arc;
use std::net::SocketAddr;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Sets a port to expose the web server on
    #[arg(short, long, default_value_t = 3000)]
    port: u16,

    /// Sets the api key to authenticate against
    #[arg(long, env = "SRF_API_KEY")]
    api_key: String,
}

#[derive(Clone)]
struct AppState {
    api_key: Arc<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let state = AppState {
        api_key: Arc::new(args.api_key),
    };

    let app = Router::new().route("/", get(root)).route(
        "/filter/{*url}",
        get(filter_feed).route_layer(middleware::from_fn_with_state(state.clone(), auth)),
    );
    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Web Server to filter out premium substack posts from RSS feed\n\nUsage: {url}/filter/{rss_feed_url}?API_KEY={API_KEY}"
}

async fn filter_feed(Path(url): Path<String>) -> Result<impl IntoResponse, AppError> {
    let feed_content = fetch_feed(&url).await?;
    let mut channel = Channel::read_from(feed_content.as_bytes())
        .map_err(|e| AppError::ParseError(e.to_string()))?;

    let filtered_items = filter_items(channel.items.clone());

    channel.set_items(filtered_items);

    let mut buffer = Vec::new();
    channel
        .write_to(&mut buffer)
        .map_err(|e| AppError::BuildError(e.to_string()))?;

    let body = String::from_utf8(buffer).map_err(|e| AppError::BuildError(e.to_string()))?;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/xml; charset=utf-8".parse().unwrap(),
    );

    Ok((headers, body))
}

fn filter_items(items: Vec<Item>) -> Vec<Item> {
    items
        .iter()
        .filter(|item| {
            if let Some(content) = item.content() {
                !content.contains("Read more")
            } else {
                true
            }
        })
        .cloned()
        .collect()
}

async fn fetch_feed(url: &str) -> Result<String, AppError> {
    let client = Client::builder()
        .build()
        .map_err(|e| AppError::FetchError(e.to_string()))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| AppError::FetchError(e.to_string()))?;

    if !response.status().is_success() {
        return Err(AppError::FetchError(format!(
            "HTTP {}: Failed to fetch feed",
            response.status()
        )));
    }

    response
        .text()
        .await
        .map_err(|e| AppError::FetchError(e.to_string()))
}

async fn auth(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let uri_api_key = match Query::<UriApiKey>::try_from_uri(req.uri()) {
        Ok(Query(uri_api_key)) => uri_api_key,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Missing API_KEY").into_response(),
    };

    if uri_api_key.api_key == *state.api_key {
        next.run(req).await
    } else {
        (StatusCode::UNAUTHORIZED, "Invalid API_KEY").into_response()
    }
}

#[derive(Deserialize)]
struct UriApiKey {
    #[serde(rename = "API_KEY")]
    api_key: String,
}

#[derive(Debug)]
enum AppError {
    FetchError(String),
    ParseError(String),
    BuildError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::FetchError(msg) => (StatusCode::BAD_REQUEST, format!("Fetch Error: {}", msg)),
            AppError::ParseError(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Fetch Error: {}", msg),
            ),
            AppError::BuildError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Fetch Error: {}", msg),
            ),
        };

        (status, message).into_response()
    }
}
