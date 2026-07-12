use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

#[derive(Deserialize)]
pub struct GoogleLoginRequest {
    pub access_token: String, // Токен от Supabase после Google OAuth
}

#[derive(Deserialize)]
pub struct EmailLoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub user_id: String,
    pub email: String,
}

// Вход через Google (через Supabase Auth)
pub async fn google_login(
    State(state): State<AppState>,
    Json(req): Json<GoogleLoginRequest>,
) -> Json<AuthResponse> {
    // Здесь ты получаешь user_id от Supabase
    // и создаешь/обновляешь пользователя в своей БД
    Json(AuthResponse {
        user_id: "google-user-id".to_string(),
        email: "user@gmail.com".to_string(),
    })
}