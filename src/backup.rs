use crate::AppState;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub async fn start_backup_monitor(state: AppState) {
    let backup_interval = Duration::from_secs(60);
    let inactivity_threshold = Duration::from_secs(300);
    
    loop {
        tokio::time::sleep(backup_interval).await;
        
        // Копируем нужные данные ДО await
        let (last_activity, current_challenge, difficulty) = {
            let mining_state = state.mining_state.lock().unwrap();
            (
                mining_state.last_activity,
                mining_state.current_challenge.clone(),
                mining_state.difficulty,
            )
        }; // Guard освобождается здесь
        
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
    let db_path = "webtab_mining.db";
    if !std::path::Path::new(db_path).exists() {
        return Ok(());
    }
    
    let db_content = tokio::fs::read(db_path).await?;
    let encoded = STANDARD.encode(&db_content); // Новый синтаксис
    
    let github_token = std::env::var("GITHUB_TOKEN")
        .expect("GITHUB_TOKEN не установлен в .env");
    
    let client = reqwest::Client::new();
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
            let decoded = STANDARD.decode(content)?; // Новый синтаксис
            tokio::fs::write("webtab_mining.db", decoded).await?;
            println!("✅ База данных скачана из GitHub");
        }
    } else {
        println!("ℹ️ База данных не найдена на GitHub. Будет создана новая.");
    }
    
    Ok(())
}