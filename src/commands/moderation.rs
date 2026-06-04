use crate::types::{Context, Error};
use crate::utils::embeds::send_embed;
use serenity::model::user::User;
use serenity::builder::GetMessages;

#[poise::command(slash_command, prefix_command, required_permissions = "KICK_MEMBERS", category = "Moderation")]
pub async fn kick(
    ctx: Context<'_>, 
    #[description = "User to kick"] user: User,
    #[description = "Reason for kicking"] reason: Option<String>
) -> Result<(), Error> {
    let reason = reason.unwrap_or_else(|| "No reason provided.".to_string());

    if let Some(guild_id) = ctx.guild_id() {
        match guild_id.kick_with_reason(ctx.http(), user.id, &reason).await {
            Ok(_) => {
                send_embed(ctx, "Moderation: Kick", &format!("User {} has been kicked.\nReason: {}", user.name, reason), 0x2b2d31).await?;
            },
            Err(e) => {
                send_embed(ctx, "Error", &format!("Failed to kick user: {}", e), 0xED4245).await?;
            }
        }
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, required_permissions = "BAN_MEMBERS", category = "Moderation")]
pub async fn ban(
    ctx: Context<'_>, 
    #[description = "User to ban"] user: User,
    #[description = "Reason for banning"] reason: Option<String>
) -> Result<(), Error> {
    let reason = reason.unwrap_or_else(|| "No reason provided.".to_string());

    if let Some(guild_id) = ctx.guild_id() {
        match guild_id.ban_with_reason(ctx.http(), user.id, 0, &reason).await {
            Ok(_) => {
                send_embed(ctx, "Moderation: Ban", &format!("User {} has been banned.\nReason: {}", user.name, reason), 0x2b2d31).await?;
            },
            Err(e) => {
                send_embed(ctx, "Error", &format!("Failed to ban user: {}", e), 0xED4245).await?;
            }
        }
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, required_permissions = "BAN_MEMBERS", category = "Moderation")]
pub async fn unban(
    ctx: Context<'_>, 
    #[description = "ID of user to unban"] user_id: u64
) -> Result<(), Error> {
    if let Some(guild_id) = ctx.guild_id() {
        let uid = serenity::model::id::UserId::new(user_id);
        match guild_id.unban(ctx.http(), uid).await {
            Ok(_) => {
                send_embed(ctx, "Moderation: Unban", &format!("User ID {} has been unbanned.", user_id), 0x2b2d31).await?;
            },
            Err(e) => {
                send_embed(ctx, "Error", &format!("Failed to unban user: {}", e), 0xED4245).await?;
            }
        }
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, required_permissions = "MANAGE_MESSAGES", category = "Moderation")]
pub async fn purge(
    ctx: Context<'_>, 
    #[description = "Number of messages to delete (1-100)"] amount: u8
) -> Result<(), Error> {
    if amount == 0 || amount > 100 {
        send_embed(ctx, "Error", "Please provide a number between 1 and 100.", 0xED4245).await?;
        return Ok(());
    }

    let channel = ctx.channel_id();
    let builder = GetMessages::new().limit(amount);
    
    let messages = match channel.messages(ctx.http(), builder).await {
        Ok(msgs) => msgs,
        Err(e) => {
            send_embed(ctx, "Error", &format!("Failed to fetch messages: {}", e), 0xED4245).await?;
            return Ok(());
        }
    };
    
    if messages.is_empty() {
        send_embed(ctx, "Error", "No messages to delete.", 0xED4245).await?;
        return Ok(());
    }

    let message_ids: Vec<serenity::model::id::MessageId> = messages.iter().map(|m| m.id).collect();
    
    if let Err(e) = channel.delete_messages(ctx.http(), message_ids).await {
        send_embed(ctx, "Error", &format!("Failed to delete messages: {}", e), 0xED4245).await?;
        return Ok(());
    }

    let reply = ctx.send(poise::CreateReply::default().content(format!("✅ Deleted {} messages.", amount)).ephemeral(true)).await?;
    
    // In Poise, ephemeral messages dismiss themselves client-side, 
    // or we can just send it normally and delete it later if prefix command.
    if ctx.prefix() != "/" {
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        let _ = reply.delete().await;
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, required_permissions = "MODERATE_MEMBERS", category = "Moderation")]
pub async fn timeout(
    ctx: Context<'_>, 
    #[description = "User to timeout"] user: User,
    #[description = "Duration in minutes"] minutes: i64
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    
    let timestamp = serenity::model::Timestamp::from_unix_timestamp(serenity::model::Timestamp::now().unix_timestamp() + (minutes * 60)).unwrap();
    
    let builder = serenity::builder::EditMember::new().disable_communication_until(timestamp.to_string());

    match guild_id.edit_member(ctx.http(), user.id, builder).await {
        Ok(_) => {
            send_embed(ctx, "Moderation: Timeout", &format!("User {} has been timed out for {} minutes.", user.name, minutes), 0x2b2d31).await?;
        },
        Err(e) => {
            send_embed(ctx, "Error", &format!("Failed to timeout user: {}", e), 0xED4245).await?;
        }
    }
    
    Ok(())
}
