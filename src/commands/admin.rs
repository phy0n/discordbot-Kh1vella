use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::{
        channel::{Message, PermissionOverwriteType, PermissionOverwrite},
        permissions::Permissions,
    },
    builder::EditChannel,
};
use crate::utils::embeds::send_embed;
use crate::handler::ChatbotState;

#[command]
#[only_in(guilds)]
#[required_permissions("MANAGE_CHANNELS")]
pub async fn lock(ctx: &Context, msg: &Message) -> CommandResult {
    let everyone_role_id = serenity::model::id::RoleId::new(msg.guild_id.unwrap().get());
    
    let channel = match msg.channel_id.to_channel(&ctx.http).await {
        Ok(c) => c.guild(),
        Err(_) => None,
    };
    
    if let Some(mut c) = channel {
        let overwrite = PermissionOverwrite {
            allow: Permissions::empty(),
            deny: Permissions::SEND_MESSAGES,
            kind: PermissionOverwriteType::Role(everyone_role_id),
        };
        
        let builder = EditChannel::new().permissions(vec![overwrite]);
        if let Err(e) = c.edit(&ctx.http, builder).await {
            send_embed(ctx, msg, "Error", &format!("Failed to lock channel: {}", e), 0xED4245).await?;
            return Ok(());
        }
    }

    send_embed(ctx, msg, "Admin: Lock", "This channel has been locked.", 0x2b2d31).await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
#[required_permissions("MANAGE_CHANNELS")]
pub async fn unlock(ctx: &Context, msg: &Message) -> CommandResult {
    let everyone_role_id = serenity::model::id::RoleId::new(msg.guild_id.unwrap().get());
    
    let channel = match msg.channel_id.to_channel(&ctx.http).await {
        Ok(c) => c.guild(),
        Err(_) => None,
    };
    
    if let Some(c) = channel {
        if let Err(e) = c.delete_permission(&ctx.http, PermissionOverwriteType::Role(everyone_role_id)).await {
            send_embed(ctx, msg, "Error", &format!("Failed to unlock channel: {}", e), 0xED4245).await?;
            return Ok(());
        }
    }

    send_embed(ctx, msg, "Admin: Unlock", "This channel has been unlocked.", 0x2b2d31).await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
#[required_permissions("MANAGE_CHANNELS")]
pub async fn slowmode(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let seconds: u16 = match args.single() {
        Ok(sec) => sec,
        Err(_) => {
            send_embed(ctx, msg, "Error", "Please provide the slowmode duration in seconds (0 to disable).", 0xED4245).await?;
            return Ok(());
        }
    };
    
    if seconds > 21600 {
        send_embed(ctx, msg, "Error", "Slowmode cannot exceed 21600 seconds (6 hours).", 0xED4245).await?;
        return Ok(());
    }

    let channel = match msg.channel_id.to_channel(&ctx.http).await {
        Ok(c) => c.guild(),
        Err(_) => None,
    };
    
    if let Some(mut c) = channel {
        let builder = EditChannel::new().rate_limit_per_user(seconds);
        if let Err(e) = c.edit(&ctx.http, builder).await {
            send_embed(ctx, msg, "Error", &format!("Failed to set slowmode: {}", e), 0xED4245).await?;
            return Ok(());
        }
    }

    let status_text = if seconds == 0 {
        "Slowmode has been disabled.".to_string()
    } else {
        format!("Slowmode set to {} seconds.", seconds)
    };

    send_embed(ctx, msg, "Admin: Slowmode", &status_text, 0x2b2d31).await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn chatbot(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let action = args.single::<String>().unwrap_or_default().to_lowercase();
    
    if action == "enable" || action == "on" {
        let mut data = ctx.data.write().await;
        if let Some(state) = data.get_mut::<ChatbotState>() {
            *state.write().await = true;
        }
        send_embed(ctx, msg, "Chatbot AI", "✅ Gemini AI Chatbot has been **ENABLED**.\nThe bot will now reply to tags and replies.", 0x2b2d31).await?;
    } else if action == "disable" || action == "off" {
        let mut data = ctx.data.write().await;
        if let Some(state) = data.get_mut::<ChatbotState>() {
            *state.write().await = false;
        }
        send_embed(ctx, msg, "Chatbot AI", "❌ Gemini AI Chatbot has been **DISABLED**.", 0x2b2d31).await?;
    } else {
        send_embed(ctx, msg, "Error", "Usage: `kh!chatbot enable` or `kh!chatbot disable`", 0xED4245).await?;
    }
    
    Ok(())
}
