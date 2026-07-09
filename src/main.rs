use axum::{routing::get, Router};
use std::net::SocketAddr;
use dotenv::dotenv;

mod database;

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    tracing_subscriber::fmt::init();
    
    let pool = database::init().await;
    
    let app = Router::new()
        .route("/", get(|| async { "Hello" }));
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}