use crate::{
    errors::AppError,
    feed::{fetch_feed, filter_items},
    srf_middleware::auth,
    state::AppState,
};
use axum::{
    Router,
    extract::{Path, Query, Request, State},
    http::{HeaderMap, header},
    middleware::{self, Next},
    response::IntoResponse,
    routing::get,
};
use clap::Parser;
use reqwest::Client;
use rss::{Channel, Item};
use std::net::SocketAddr;
use std::sync::Arc;

pub async fn root() -> &'static str {
    "Web Server to filter out premium substack posts from RSS feed\n\nUsage: {url}/filter/{rss_feed_url}?API_KEY={API_KEY}"
}

pub async fn filter_feed(Path(url): Path<String>) -> Result<impl IntoResponse, AppError> {
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
