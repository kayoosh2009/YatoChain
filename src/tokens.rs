use axum::{extract::State, Json, http::{StatusCode, HeaderMap}};
use serde::{Deserialize, Serialize};
use reqwest::header::{HeaderValue};
use crate::auth::AppState;

#[derive(Deserialize)]
pub struct CreateTokenRequest {
    pub name: String,
    pub code: String,
    pub description: String,
    pub link: String,
    pub image_url: String,
    pub initial_price: f64, // Стоимость в Y$
}

#[derive(Serialize)]
pub struct CreateTokenResponse {
    pub message: String,
    pub token_id: String,
}

pub async fn create_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateTokenRequest>,
) -> Result<Json<CreateTokenResponse>, StatusCode> {
    
    // 1. Формируем URL для Supabase REST API
    let url = format!("{}/rest/v1/tokens", state.supabase.url);
    
    // 2. Настраиваем заголовки для Supabase
    let mut req_headers = reqwest::header::HeaderMap::new();
    req_headers.insert("apikey", HeaderValue::from_str(&state.supabase.anon_key).unwrap());
    req_headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    // Эта настройка говорит Supabase вернуть нам только что созданную строку
    req_headers.insert("Prefer", HeaderValue::from_static("return=representation")); 

    // 3. Формируем тело запроса
    let body = serde_json::json!({
        "name": req.name,
        "code": req.code,
        "description": req.description,
        "link": req.link,
        "image_url": req.image_url,
        "current_price": req.initial_price,
        // Пока creator_id пустой, на следующем шаге мы научимся доставать user_id из токена авторизации
        "creator_id": null 
    });

    // 4. Отправляем POST-запрос в Supabase
    let res = state.supabase.client.post(&url)
        .headers(req_headers)
        .json(&body)
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if res.status().is_success() {
        // Парсим ответ (Supabase вернет массив с одним объектом)
        let created_tokens: Vec<serde_json::Value> = res.json().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let token_id = created_tokens.first()
            .and_then(|v| v["id"].as_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(Json(CreateTokenResponse {
            message: "Токен успешно создан".to_string(),
            token_id,
        }))
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}