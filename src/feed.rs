use crate::{
    errors::AppError,
    routes::{filter_feed, root},
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

pub fn filter_items(items: Vec<Item>) -> Vec<Item> {
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

pub async fn fetch_feed(url: &str) -> Result<String, AppError> {
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
