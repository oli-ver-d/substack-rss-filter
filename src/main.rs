use axum::{
    Router,
    extract::Path,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use reqwest::Client;
use rss::{Channel, Item};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/filter/{*url}", get(filter_feed));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Web Server to filter out premium substack posts from RSS feed\n\nUsage: {url}/filter/{rss_feed_url}"
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
