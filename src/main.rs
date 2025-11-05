use crate::{
    routes::{filter_feed, root},
    srf_middleware::auth,
    state::AppState,
};
use axum::{
    Router,
    middleware::{self},
    routing::get,
};
use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;

mod errors;
mod feed;
mod routes;
mod srf_middleware;
mod state;

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
