use reqwest::Client;

#[derive(Clone)]
pub struct SupabaseClient {
    pub client: Client,
    pub url: String,
    pub anon_key: String,
}

impl SupabaseClient {
    pub fn new(url: String, anon_key: String) -> Self {
        Self {
            client: Client::new(),
            url,
            anon_key,
        }
    }
}

pub async fn init() -> SupabaseClient {
    let supabase_url = std::env::var("SUPABASE_URL")
        .expect("SUPABASE_URL must be set");
    let supabase_anon_key = std::env::var("SUPABASE_ANON_KEY")
        .expect("SUPABASE_ANON_KEY must be set");

    tracing::info!("Supabase client initialized");
    SupabaseClient::new(supabase_url, supabase_anon_key)
}