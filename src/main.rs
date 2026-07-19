use axum::{routing::post, Router};
use std::net::SocketAddr;
use dotenv::dotenv;
use tower_http::services::ServeDir; 

mod database;
mod auth;
mod tokens;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let supabase_client = database::init().await;
    let state = auth::AppState { supabase: supabase_client };

    let app = Router::new()
        .route("/auth/register", post(auth::email_register))
        .route("/auth/email", post(auth::email_login))
        .route("/tokens/create", axum::routing::post(tokens::create_token))
        .fallback_service(ServeDir::new("static"))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}