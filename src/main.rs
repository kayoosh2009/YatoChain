use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Router,
};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tower_http::services::ServeDir;

mod backup;
mod mining;

#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<Connection>>,
    mining_state: Arc<Mutex<MiningState>>,
}

struct MiningState {
    current_challenge: String,
    difficulty: usize,
    active_users: usize,
    last_activity: u64,
}

#[derive(Serialize, Deserialize)]
struct WalletRequest {
    wallet_address: String,
}

#[derive(Serialize, Deserialize)]
struct MiningResponse {
    challenge: String,
    difficulty: usize,
}

#[derive(Serialize, Deserialize)]
struct SolutionRequest {
    wallet_address: String,
    nonce: u64,
    hash: String,
}

#[derive(Serialize, Deserialize)]
struct SolutionResponse {
    success: bool,
    tokens_earned: f64,
    message: String,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    
    // Инициализация базы данных
    let conn = initialize_database().expect("Failed to initialize database");
    
    let state = AppState {
        db: Arc::new(Mutex::new(conn)),
        mining_state: Arc::new(Mutex::new(MiningState {
            current_challenge: mining::generate_challenge(),
            difficulty: 3,
            active_users: 0,
            last_activity: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })),
    };
    
    // Запуск фонового процесса для бэкапов
    let backup_state = state.clone();
    tokio::spawn(async move {
        backup::start_backup_monitor(backup_state).await;
    });
    
    // Настройка роутов
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/register", post(register_wallet))
        .route("/api/mining", get(get_mining_task))
        .route("/api/solution", post(submit_solution))
        .route("/api/stats", get(get_stats))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);
    
    // Запуск сервера
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind");
    
    println!("🚀 WebTab Mining запущен на http://localhost:3000");
    
    axum::serve(listener, app)
        .await
        .expect("Server failed");
}

fn initialize_database() -> Result<Connection, rusqlite::Error> {
    let db_path = "webtab_mining.db";
    
    // Проверка существования БД
    let db_exists = std::path::Path::new(db_path).exists();
    
    if !db_exists {
        println!("📥 База данных не найдена. Скачиваем из GitHub...");
        // TODO: Реализовать скачивание из GitHub
        // backup::download_from_github().await;
    }
    
    let conn = Connection::open(db_path)?;
    
    // Создание таблиц
    conn.execute(
        "CREATE TABLE IF NOT EXISTS wallets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            address TEXT UNIQUE NOT NULL,
            total_tokens REAL DEFAULT 0.0,
            hashes_found INTEGER DEFAULT 0,
            created_at INTEGER NOT NULL,
            last_active INTEGER NOT NULL
        )",
        [],
    )?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS mining_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            wallet_id INTEGER NOT NULL,
            challenge TEXT NOT NULL,
            nonce INTEGER NOT NULL,
            hash TEXT NOT NULL,
            tokens_earned REAL NOT NULL,
            timestamp INTEGER NOT NULL,
            FOREIGN KEY (wallet_id) REFERENCES wallets(id)
        )",
        [],
    )?;
    
    println!("✅ База данных инициализирована");
    Ok(conn)
}

async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

async fn register_wallet(
    State(state): State<AppState>,
    Json(payload): Json<WalletRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let db = state.db.lock().unwrap();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let result = db.execute(
        "INSERT OR IGNORE INTO wallets (address, created_at, last_active) VALUES (?1, ?2, ?2)",
        params![payload.wallet_address, now],
    );
    
    match result {
        Ok(_) => {
            // Увеличиваем счетчик юзеров и пересчитываем сложность
            let mut mining_state = state.mining_state.lock().unwrap();
            mining_state.active_users += 1;
            mining_state.difficulty = mining::calculate_difficulty(mining_state.active_users);
            
            println!("👤 Юзеров: {}, Сложность: {}", mining_state.active_users, mining_state.difficulty);

            Ok(Json(serde_json::json!({
                "success": true,
                "message": "Кошелек зарегистрирован"
            })))
        }
}

async fn get_mining_task(
    State(state): State<AppState>,
) -> Json<MiningResponse> {
    let mining_state = state.mining_state.lock().unwrap();
    
    // Обновляем время последней активности
    drop(mining_state);
    let mut mining_state = state.mining_state.lock().unwrap();
    mining_state.last_activity = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    mining_state.active_users += 1;
    
    Json(MiningResponse {
        challenge: mining_state.current_challenge.clone(),
        difficulty: mining_state.difficulty,
    })
}

async fn submit_solution(
    State(state): State<AppState>,
    Json(payload): Json<SolutionRequest>,
) -> Result<Json<SolutionResponse>, StatusCode> {
    let mining_state = state.mining_state.lock().unwrap();
    
    // Проверяем правильность решения
    let expected_hash = mining::compute_hash(
        &mining_state.current_challenge,
        payload.nonce,
    );
    
    if expected_hash != payload.hash {
        return Ok(Json(SolutionResponse {
            success: false,
            tokens_earned: 0.0,
            message: "Неверный хеш".to_string(),
        }));
    }
    
    // Проверяем сложность (количество начальных нулей)
    let leading_zeros = payload.hash.chars().take_while(|&c| c == '0').count();
    if leading_zeros < mining_state.difficulty {
        return Ok(Json(SolutionResponse {
            success: false,
            tokens_earned: 0.0,
            message: "Сложность недостаточна".to_string(),
        }));
    }
    
    // Вычисляем токены
    let tokens_earned = payload.hash.len() as f64 * 0.00001;
    
    // Обновляем базу данных
    let db = state.db.lock().unwrap();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Получаем или создаем кошелек
    let wallet_id: i64 = db
        .query_row(
            "SELECT id FROM wallets WHERE address = ?1",
            params![payload.wallet_address],
            |row| row.get(0),
        )
        .unwrap_or(0);
    
    if wallet_id == 0 {
        db.execute(
            "INSERT INTO wallets (address, created_at, last_active) VALUES (?1, ?2, ?2)",
            params![payload.wallet_address, now],
        )
        .unwrap();
    }
    
    let wallet_id: i64 = db
        .query_row(
            "SELECT id FROM wallets WHERE address = ?1",
            params![payload.wallet_address],
            |row| row.get(0),
        )
        .unwrap();
    
    // Обновляем статистику кошелька
    db.execute(
        "UPDATE wallets SET total_tokens = total_tokens + ?1, hashes_found = hashes_found + 1, last_active = ?2 WHERE id = ?3",
        params![tokens_earned, now, wallet_id],
    )
    .unwrap();
    
    // Записываем в историю
    db.execute(
        "INSERT INTO mining_history (wallet_id, challenge, nonce, hash, tokens_earned, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![wallet_id, mining_state.current_challenge, payload.nonce, payload.hash, tokens_earned, now],
    )
    .unwrap();
    
    // Генерируем новый challenge
    drop(mining_state);
    let mut mining_state = state.mining_state.lock().unwrap();
    mining_state.current_challenge = mining::generate_challenge();
    
    Ok(Json(SolutionResponse {
        success: true,
        tokens_earned,
        message: "Хеш найден!".to_string(),
    }))
}

async fn get_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let db = state.db.lock().unwrap();
    
    let total_wallets: i64 = db
        .query_row("SELECT COUNT(*) FROM wallets", [], |row| row.get(0))
        .unwrap();
    
    let total_hashes: i64 = db
        .query_row("SELECT SUM(hashes_found) FROM wallets", [], |row| row.get(0))
        .unwrap_or(0);
    
    let mining_state = state.mining_state.lock().unwrap();
    
    Ok(Json(serde_json::json!({
        "total_wallets": total_wallets,
        "total_hashes": total_hashes,
        "active_users": mining_state.active_users,
        "current_difficulty": mining_state.difficulty,
        "current_challenge": mining_state.current_challenge
    })))
}