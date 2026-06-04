use std::sync::Arc;

pub struct Data {
    pub chatbot_enabled: Arc<tokio::sync::RwLock<bool>>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
