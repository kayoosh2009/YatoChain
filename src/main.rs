use axum::{routing::{get, post}, Router};
use std::net::SocketAddr;
use dotenv::dotenv;

mod database;
mod auth;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();
    
    let pool = database::init().await;
    let state = auth::AppState { db: pool };
    
    let app = Router::new()
        .route("/", get(|| async { "Hello" }))
        .route("/auth/google", post(auth::google_login))
        .route("/auth/email", post(auth::email_login))
        .with_state(state);
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}