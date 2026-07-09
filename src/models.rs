use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub telegram_username: Option<String>,
    pub telegram_id: Option<i64>,
    pub is_verified: bool,
    pub is_suspicious: bool,
    pub is_banned: bool,
    pub daily_exp: i32,
    pub total_exp: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Token {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub ticker: String,
    pub name: String,
    pub total_supply: i64,
    pub current_price: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Balance {
    pub user_id: Uuid,
    pub token_id: Uuid,
    pub amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_id: Uuid,
    pub order_type: String, // "buy" or "sell"
    pub price: f64,
    pub amount: i64,
    pub filled_amount: i64,
    pub status: String, // "open", "filled", "cancelled"
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DailyStats {
    pub user_id: Uuid,
    pub date: chrono::NaiveDate,
    pub exp_earned: i32,
    pub trades_count: i32,
}

// DTO для API
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTokenRequest {
    pub ticker: String,
    pub name: String,
    pub total_supply: i64,
}

#[derive(Debug, Deserialize)]
pub struct PlaceOrderRequest {
    pub token_id: Uuid,
    pub order_type: String,
    pub price: f64,
    pub amount: i64,
}