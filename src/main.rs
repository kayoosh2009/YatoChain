use axum::{routing::{get, post}, Router};
use std::net::SocketAddr;
use dotenv::dotenv;
// Добавляем импорт для раздачи статики
use tower_http::services::ServeDir; 

mod database;
mod auth;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let supabase_client = database::init().await;
    let state = auth::AppState { supabase: supabase_client };

    let app = Router::new()
        .route("/", get(|| async { "Hello" }))
        .route("/auth/google", post(auth::google_login))
        .route("/auth/email", post(auth::email_login))
        // Добавляем раздачу всех файлов из папки "static" по пути "/static"
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}