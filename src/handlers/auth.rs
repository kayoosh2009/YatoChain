use axum::{extract::State, Json};
use crate::{AppState, AppError, models::*};
use uuid::Uuid;
use chrono::Utc;

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Проверка, что email не занят
    let existing = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM users WHERE email = $1"
    )
    .bind(&req.email)
    .fetch_optional(&state.db)
    .await?;

    if existing.is_some() {
        return Err(AppError::BadRequest("Email already registered".to_string()));
    }

    // Хэширование пароля
    let password_hash = bcrypt::hash(&req.password, 10)
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?;

    // Создание пользователя
    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, is_verified, is_suspicious, is_banned, daily_exp, total_exp, created_at, updated_at)
         VALUES ($1, $2, $3, false, false, false, 0, 0, $4, $4)"
    )
    .bind(user_id)
    .bind(&req.email)
    .bind(&password_hash)
    .bind(Utc::now())
    .execute(&state.db)
    .await?;

    // Генерация JWT
    let token = crate::auth::generate_jwt(user_id, &state.jwt_secret)?;

    Ok(Json(AuthResponse { token, user_id }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Поиск пользователя
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(&req.email)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

    // Проверка пароля
    let valid = bcrypt::verify(&req.password, &user.password_hash)
        .map_err(|e| AppError::Internal(format!("Failed to verify password: {}", e)))?;

    if !valid {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    // Генерация JWT
    let token = crate::auth::generate_jwt(user.id, &state.jwt_secret)?;

    Ok(Json(AuthResponse { token, user_id: user.id }))
}