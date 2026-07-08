use crate::AppState;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub async fn start_backup_monitor(state: AppState) {
    let backup_interval = Duration::from_secs(60); // Проверяем каждую минуту
    let inactivity_threshold = Duration::from_secs(300); // 5 минут неактивности
    
    loop {
        tokio::time::sleep(backup_interval).await;
        
        let mining_state = state.mining_state.lock().unwrap();
        let last_activity = mining_state.last_activity;
        drop(mining_state);
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if now - last_activity >= inactivity_threshold.as_secs() {
            println!("💾 Обнаружена неактивность. Создаем бэкап...");
            if let Err(e) = create_backup_and_push().await {
                eprintln!("Ошибка бэкапа: {}", e);
            }
        }
    }
}

async fn create_backup_and_push() -> Result<(), Box<dyn std::error::Error>> {
    // Читаем файл БД
    let db_path = "webtab_mining.db";
    if !std::path::Path::new(db_path).exists() {
        return Ok(());
    }
    
    let db_content = tokio::fs::read(db_path).await?;
    let encoded = base64::encode(&db_content);
    
    // Получаем токен из .env
    let github_token = std::env::var("GITHUB_TOKEN")
        .expect("GITHUB_TOKEN не установлен в .env");
    
    let client = reqwest::Client::new();
    
    // Получаем текущий SHA файла (если существует)
    let repo = "kayoosh2009/WebTab-Mining";
    let path = "webtab_mining.db";
    let branch = "main";
    
    let get_url = format!(
        "https://api.github.com/repos/{}/contents/{}?ref={}",
        repo, path, branch
    );
    
    let sha = client
        .get(&get_url)
        .header("Authorization", format!("token {}", github_token))
        .header("User-Agent", "WebTab-Mining")
        .send()
        .await?
        .json::<serde_json::Value>()
        .await
        .ok()
        .and_then(|v| v.get("sha").and_then(|s| s.as_str().map(String::from)));
    
    // Создаем или обновляем файл
    let mut body = serde_json::json!({
        "message": format!("Auto backup {}", chrono::Utc::now().to_rfc3339()),
        "content": encoded,
        "branch": branch
    });
    
    if let Some(sha) = sha {
        body["sha"] = serde_json::Value::String(sha);
    }
    
    let put_url = format!(
        "https://api.github.com/repos/{}/contents/{}",
        repo, path
    );
    
    let response = client
        .put(&put_url)
        .header("Authorization", format!("token {}", github_token))
        .header("User-Agent", "WebTab-Mining")
        .json(&body)
        .send()
        .await?;
    
    if response.status().is_success() {
        println!("✅ Бэкап успешно загружен на GitHub");
    } else {
        println!("❌ Ошибка загрузки бэкапа: {}", response.status());
    }
    
    Ok(())
}

pub async fn download_from_github() -> Result<(), Box<dyn std::error::Error>> {
    let github_token = std::env::var("GITHUB_TOKEN")
        .expect("GITHUB_TOKEN не установлен в .env");
    
    let client = reqwest::Client::new();
    let repo = "kayoosh2009/WebTab-Mining";
    let path = "webtab_mining.db";
    let branch = "main";
    
    let url = format!(
        "https://api.github.com/repos/{}/contents/{}?ref={}",
        repo, path, branch
    );
    
    let response = client
        .get(&url)
        .header("Authorization", format!("token {}", github_token))
        .header("User-Agent", "WebTab-Mining")
        .send()
        .await?;
    
    if response.status().is_success() {
        let json: serde_json::Value = response.json().await?;
        
        if let Some(content) = json.get("content").and_then(|c| c.as_str()) {
            let decoded = base64::decode(content)?;
            tokio::fs::write("webtab_mining.db", decoded).await?;
            println!("✅ База данных скачана из GitHub");
        }
    } else {
        println!("ℹ️ База данных не найдена на GitHub. Будет создана новая.");
    }
    
    Ok(())
}