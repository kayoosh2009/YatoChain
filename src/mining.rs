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

#[allow(dead_code)]
pub fn calculate_difficulty(active_users: usize) -> usize {
    match active_users {
        0..=5 => 2,
        6..=20 => 3,
        21..=50 => 4,
        _ => 5,
    }
}