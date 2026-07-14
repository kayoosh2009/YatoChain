use axum::{extract::State, response::Redirect, Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use crate::database::SupabaseClient;

#[derive(Clone)]
pub struct AppState {
    pub supabase: SupabaseClient,
}

#[derive(Deserialize)]
pub struct GoogleLoginRequest {
    pub access_token: String, // JWT токен от Supabase после Google OAuth
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

// Вспомогательные структуры для парсинга ответов Supabase
#[derive(Deserialize)]
struct SupabaseUserResponse {
    id: String,
    email: String,
}

#[derive(Deserialize)]
struct SupabaseAuthResponse {
    access_token: String,
    user: SupabaseUserResponse,
}

// Вход через Google (проверка токена)
pub async fn google_login(
    State(state): State<AppState>,
    Json(req): Json<GoogleLoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let url = format!("{}/auth/v1/user", state.supabase.url);
    
    let mut headers = HeaderMap::new();
    headers.insert("apikey", HeaderValue::from_str(&state.supabase.anon_key).unwrap());
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", req.access_token)).unwrap());

    let res = state.supabase.client.get(&url)
        .headers(headers)
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if res.status().is_success() {
        let user: SupabaseUserResponse = res.json().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Json(AuthResponse {
            user_id: user.id,
            email: user.email,
        }))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

// Вход через email/password
pub async fn email_login(
    State(state): State<AppState>,
    Json(req): Json<EmailLoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let url = format!("{}/auth/v1/token?grant_type=password", state.supabase.url);
    
    let mut headers = HeaderMap::new();
    headers.insert("apikey", HeaderValue::from_str(&state.supabase.anon_key).unwrap());
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));

    let body = serde_json::json!({
        "email": req.email,
        "password": req.password
    });

    let res = state.supabase.client.post(&url)
        .headers(headers)
        .json(&body)
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if res.status().is_success() {
        let auth_res: SupabaseAuthResponse = res.json().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Json(AuthResponse {
            user_id: auth_res.user.id,
            email: auth_res.user.email,
        }))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn google_start() -> Result<Redirect, StatusCode> {
    let supabase_url = std::env::var("SUPABASE_URL")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Важно: этот URL должен быть добавлен в Supabase Dashboard -> Authentication -> URL Configuration -> Redirect URLs
    let redirect_to = "http://127.0.0.1:3000/lets-start.html";
    
    let auth_url = format!(
        "{}/auth/v1/authorize?provider=google&redirect_to={}",
        supabase_url, redirect_to
    );
    
    Ok(Redirect::to(&auth_url))
}