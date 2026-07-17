use axum::{extract::State, Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use reqwest::header::{HeaderMap, HeaderValue};
use crate::database::SupabaseClient;

#[derive(Clone)]
pub struct AppState {
    pub supabase: SupabaseClient,
}

#[derive(Deserialize)]
pub struct EmailLoginRequest {
    pub email: String,
    pub password: String,
}
#[derive(Deserialize)]
pub struct EmailRegisterRequest {
    pub email: String,
    pub password: String,
    pub nickname: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub user_id: String,
    pub email: String,
    pub access_token: Option<String>, // Делаем опциональным
    pub message: Option<String>, 
}

// Вспомогательные структуры для парсинга ответов Supabase
#[derive(Deserialize)]
struct SupabaseUserResponse {
    id: String,
    email: String,
}

#[derive(Deserialize)]
struct SupabaseAuthResponse {
    access_token: Option<String>,
    user: SupabaseUserResponse,
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
            access_token: auth_res.access_token, // Просто присваиваем, так как это уже Option<String>
            message: None,
        }))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn email_register(
    State(state): State<AppState>,
    Json(req): Json<EmailRegisterRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let url = format!("{}/auth/v1/signup", state.supabase.url);
    let mut headers = HeaderMap::new();
    headers.insert("apikey", HeaderValue::from_str(&state.supabase.anon_key).unwrap());
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    
    let body = serde_json::json!({
        "email": req.email,
        "password": req.password,
        "data": { "nickname": req.nickname }
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
            access_token: None,                        // <-- ДОБАВИТЬ
            message: Some("Письмо для подтверждения отправлено на вашу почту.".to_string()), // <-- ДОБАВИТЬ
        }))
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}