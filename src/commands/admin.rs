use crate::types::{Context, Error};
use crate::utils::embeds::send_embed;
use serenity::model::{
    channel::{PermissionOverwriteType, PermissionOverwrite},
    permissions::Permissions,
};
use serenity::builder::EditChannel;

#[poise::command(slash_command, prefix_command, required_permissions = "MANAGE_CHANNELS", category = "Admin")]
pub async fn lock(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let everyone_role_id = serenity::model::id::RoleId::new(guild_id.get());
    
    let channel = match ctx.channel_id().to_channel(ctx.http()).await {
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
        if let Err(e) = c.edit(ctx.http(), builder).await {
            send_embed(ctx, "Error", &format!("Failed to lock channel: {}", e), 0xED4245).await?;
            return Ok(());
        }
    }

    send_embed(ctx, "Admin: Lock", "This channel has been locked.", 0x2b2d31).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, required_permissions = "MANAGE_CHANNELS", category = "Admin")]
pub async fn unlock(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let everyone_role_id = serenity::model::id::RoleId::new(guild_id.get());
    
    let channel = match ctx.channel_id().to_channel(ctx.http()).await {
        Ok(c) => c.guild(),
        Err(_) => None,
    };
    
    if let Some(c) = channel {
        if let Err(e) = c.delete_permission(ctx.http(), PermissionOverwriteType::Role(everyone_role_id)).await {
            send_embed(ctx, "Error", &format!("Failed to unlock channel: {}", e), 0xED4245).await?;
            return Ok(());
        }
    }

    send_embed(ctx, "Admin: Unlock", "This channel has been unlocked.", 0x2b2d31).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, required_permissions = "MANAGE_CHANNELS", category = "Admin")]
pub async fn slowmode(
    ctx: Context<'_>, 
    #[description = "Duration in seconds (0 to disable)"] seconds: u16
) -> Result<(), Error> {
    if seconds > 21600 {
        send_embed(ctx, "Error", "Slowmode cannot exceed 21600 seconds (6 hours).", 0xED4245).await?;
        return Ok(());
    }

    let channel = match ctx.channel_id().to_channel(ctx.http()).await {
        Ok(c) => c.guild(),
        Err(_) => None,
    };
    
    if let Some(mut c) = channel {
        let builder = EditChannel::new().rate_limit_per_user(seconds);
        if let Err(e) = c.edit(ctx.http(), builder).await {
            send_embed(ctx, "Error", &format!("Failed to set slowmode: {}", e), 0xED4245).await?;
            return Ok(());
        }
    }

    let status_text = if seconds == 0 {
        "Slowmode has been disabled.".to_string()
    } else {
        format!("Slowmode set to {} seconds.", seconds)
    };

    send_embed(ctx, "Admin: Slowmode", &status_text, 0x2b2d31).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, owners_only, category = "Admin")]
pub async fn chatbot(
    ctx: Context<'_>, 
    #[description = "Action: 'enable' or 'disable'"] action: String
) -> Result<(), Error> {
    let action = action.to_lowercase();
    
    if action == "enable" || action == "on" {
        *ctx.data().chatbot_enabled.write().await = true;
        send_embed(ctx, "Chatbot AI", "Chatbot AI has been **ENABLED**.\nThe bot will now reply to tags and replies.", 0x2b2d31).await?;
    } else if action == "disable" || action == "off" {
        *ctx.data().chatbot_enabled.write().await = false;
        send_embed(ctx, "Chatbot AI", "Chatbot AI has been **DISABLED**.", 0x2b2d31).await?;
    } else {
        send_embed(ctx, "Error", "Usage: `/chatbot enable` or `/chatbot disable`", 0xED4245).await?;
    }
    
    Ok(())
}

#[poise::command(slash_command, prefix_command, owners_only, category = "Admin")]
pub async fn status(
    ctx: Context<'_>, 
    #[description = "Type: 'playing', 'watching', 'listening', 'competing'"] activity_type: String,
    #[rest]
    #[description = "The status message"] message: String,
) -> Result<(), Error> {
    use serenity::all::ActivityData;
    use serenity::all::OnlineStatus;

    let activity = match activity_type.to_lowercase().as_str() {
        "watching" => Some(ActivityData::watching(&message)),
        "listening" => Some(ActivityData::listening(&message)),
        "competing" => Some(ActivityData::competing(&message)),
        _ => Some(ActivityData::playing(&message)),
    };

    ctx.serenity_context().set_presence(activity, OnlineStatus::Online);
    send_embed(ctx, "Admin: Status", &format!("Status updated successfully."), 0x2b2d31).await?;
    
    Ok(())
}
