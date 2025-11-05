use crate::errors::AppError;
use reqwest::Client;
use rss::Item;

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
