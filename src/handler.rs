use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{gateway::Ready, channel::Message},
};
use tracing::{info, error};
use std::sync::Arc;
use tokio::sync::RwLock;
use serenity::prelude::TypeMapKey;

use crate::utils::ai::ask_gemini;

pub struct ChatbotState;

impl TypeMapKey for ChatbotState {
    type Value = Arc<RwLock<bool>>;
}

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("System initialized and connected as {}", ready.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        let is_enabled = {
            let data = ctx.data.read().await;
            if let Some(state) = data.get::<ChatbotState>() {
                *state.read().await
            } else {
                false
            }
        };

        if is_enabled {
            let bot_id = ctx.cache.current_user().id;
            
            let is_mention = msg.mentions.iter().any(|u| u.id == bot_id);
            let is_reply = if let Some(ref_msg) = &msg.referenced_message {
                ref_msg.author.id == bot_id
            } else {
                false
            };

            if is_mention || is_reply {
                let mut prompt = msg.content.clone();
                let mention_tag1 = format!("<@{}>", bot_id);
                let mention_tag2 = format!("<@!{}>", bot_id);
                prompt = prompt.replace(&mention_tag1, "").replace(&mention_tag2, "").trim().to_string();

                if prompt.is_empty() {
                    prompt = "Halo Kh1vella!".to_string();
                }

                let _ = msg.channel_id.broadcast_typing(&ctx.http).await;

                match ask_gemini(&prompt).await {
                    Ok(reply) => {
                        let _ = msg.reply(&ctx.http, reply).await;
                    }
                    Err(e) => {
                        error!("Gemini API Error: {}", e);
                        let _ = msg.reply(&ctx.http, "Maaf, sistem AI sedang offline atau limit API tercapai.").await;
                    }
                }
            }
        }
    }
}
