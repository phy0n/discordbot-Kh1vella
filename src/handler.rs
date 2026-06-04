use serenity::client::Context as SerenityContext;
use crate::types::{Data, Error};

pub async fn event_handler(
    ctx: &SerenityContext,
    event: &serenity::all::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    if let serenity::all::FullEvent::Message { new_message: msg } = event {
        if msg.author.bot {
            return Ok(());
        }

        let is_enabled = *data.chatbot_enabled.read().await;

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
                crate::utils::ai::handle_chat(ctx, msg, data, &prompt).await;
            }
        }
    }
    
    Ok(())
}
