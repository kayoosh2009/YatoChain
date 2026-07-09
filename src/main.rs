use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use dotenv::dotenv;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Модели данных
mod models;
mod handlers;
mod error;
mod auth;

use models::*;
use error::AppError;

// Состояние приложения
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Загрузка переменных окружения
    dotenv().ok();

    // Инициализация логирования
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Подключение к базе данных
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    tracing::info!("Connected to database");

    // Запуск миграций
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    tracing::info!("Database migrations completed");

    // JWT секрет
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string());

    // Состояние приложения
    let state = AppState {
        db: pool,
        jwt_secret,
    };

    // Роуты
    let app = Router::new()
        // Публичные роуты
        .route("/health", get(health_check))
        .route("/api/register", post(handlers::auth::register))
        .route("/api/login", post(handlers::auth::login))
        
        // Защищенные роуты (требуют JWT)
        .route("/api/user/me", get(handlers::user::get_current_user))
        .route("/api/tokens", get(handlers::tokens::list_tokens))
        .route("/api/tokens", post(handlers::tokens::create_token))
        .route("/api/tokens/:id", get(handlers::tokens::get_token))
        .route("/api/trade/order", post(handlers::trading::place_order))
        .route("/api/trade/orders", get(handlers::trading::get_user_orders))
        .route("/api/trade/balance", get(handlers::trading::get_balance))
        
        // Telegram верификация
        .route("/api/telegram/generate-code", post(handlers::telegram::generate_code))
        
        .with_state(state);

    // Запуск сервера
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Server starting on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// Простой health check
async fn health_check() -> &'static str {
    "OK"
}