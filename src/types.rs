use std::sync::Arc;

pub struct Data {
    pub chatbot_enabled: Arc<tokio::sync::RwLock<bool>>,
    pub db_pool: sqlx::PgPool,
    pub chat_history: Arc<tokio::sync::RwLock<std::collections::HashMap<u64, Vec<serde_json::Value>>>>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
