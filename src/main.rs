use axum::{routing::post, Router};
use std::net::SocketAddr;
use dotenv::dotenv;
// Импортируем ServeDir для раздачи статики
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
        // API роуты
        .route("/auth/google", post(auth::google_login))
        .route("/auth/email", post(auth::email_login))
        
        // Fallback сервис: если путь не совпал с API (выше), 
        // ищем файл в папке "static". 
        // Теперь в ссылке не нужно писать /static/
        .fallback_service(ServeDir::new("static"))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}