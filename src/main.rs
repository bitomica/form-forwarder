use dotenv::dotenv;

pub mod config;
pub mod handler;
pub mod smtpclient;

use axum::{routing::post, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();
    let server_config = config::ServerConfig::default();
    let listener = TcpListener::bind(format!("0.0.0.0:{}", server_config.port))
        .await
        .unwrap();
    info!("Server listening on {}", listener.local_addr().unwrap());

    let app = Router::new().route("/", post(handler::receive_form));
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap()
}
