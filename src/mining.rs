use rand::Rng;
use sha2::{Digest, Sha256};

pub fn generate_challenge() -> String {
    let mut rng = rand::thread_rng();
    let length = rng.gen_range(16..32);
    (0..length)
        .map(|_| rng.gen_range(b'a'..=b'z') as char)
        .collect()
}

pub fn compute_hash(challenge: &str, nonce: u64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}{}", challenge, nonce));
    let result = hasher.finalize();
    hex::encode(result)
}

// Начальная сложность 3, растёт с количеством пользователей
pub fn calculate_difficulty(active_users: usize) -> usize {
    match active_users {
        0..=3 => 3,    // 0-5 пользователей: сложность 3
        4..=6 => 4,   // 6-15 пользователей: сложность 4
        7..=10 => 5,  // 16-30 пользователей: сложность 5
        11..=12 => 6,  // 31-50 пользователей: сложность 6
        _ => 7,        // 50+ пользователей: сложность 7
    }
}