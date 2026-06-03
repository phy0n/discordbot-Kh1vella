use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::{channel::Message, id::UserId, Timestamp},
    builder::GetMessages,
};
use std::time::Duration;
use crate::utils::embeds::send_embed;

#[command]
#[only_in(guilds)]
#[required_permissions("KICK_MEMBERS")]
pub async fn kick(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if msg.mentions.is_empty() {
        send_embed(ctx, msg, "Error", "You must mention a user to kick.", 0xED4245).await?;
        return Ok(());
    }

    let user_to_kick = &msg.mentions[0];
    args.advance();
    
    let reason = if args.is_empty() {
        "No reason provided.".to_string()
    } else {
        args.rest().to_string()
    };

    if let Some(guild_id) = msg.guild_id {
        match guild_id.kick_with_reason(&ctx.http, user_to_kick.id, &reason).await {
            Ok(_) => {
                send_embed(ctx, msg, "Moderation: Kick", &format!("User {} has been kicked.\nReason: {}", user_to_kick.name, reason), 0x2b2d31).await?;
            },
            Err(e) => {
                send_embed(ctx, msg, "Error", &format!("Failed to kick user: {}", e), 0xED4245).await?;
            }
        }
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
#[required_permissions("BAN_MEMBERS")]
pub async fn ban(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if msg.mentions.is_empty() {
        send_embed(ctx, msg, "Error", "You must mention a user to ban.", 0xED4245).await?;
        return Ok(());
    }

    let user_to_ban = &msg.mentions[0];
    args.advance();

    let reason = if args.is_empty() {
        "No reason provided.".to_string()
    } else {
        args.rest().to_string()
    };

    if let Some(guild_id) = msg.guild_id {
        match guild_id.ban_with_reason(&ctx.http, user_to_ban.id, 0, &reason).await {
            Ok(_) => {
                send_embed(ctx, msg, "Moderation: Ban", &format!("User {} has been banned.\nReason: {}", user_to_ban.name, reason), 0x2b2d31).await?;
            },
            Err(e) => {
                send_embed(ctx, msg, "Error", &format!("Failed to ban user: {}", e), 0xED4245).await?;
            }
        }
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
#[required_permissions("BAN_MEMBERS")]
pub async fn unban(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let user_id_val = match args.single::<u64>() {
        Ok(id) => id,
        Err(_) => {
            send_embed(ctx, msg, "Error", "You must provide a valid user ID to unban.", 0xED4245).await?;
            return Ok(());
        }
    };
    
    let user_id = UserId::new(user_id_val);

    if let Some(guild_id) = msg.guild_id {
        match guild_id.unban(&ctx.http, user_id).await {
            Ok(_) => {
                send_embed(ctx, msg, "Moderation: Unban", &format!("User ID {} has been unbanned.", user_id_val), 0x2b2d31).await?;
            },
            Err(e) => {
                send_embed(ctx, msg, "Error", &format!("Failed to unban user: {}", e), 0xED4245).await?;
            }
        }
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
#[required_permissions("MANAGE_MESSAGES")]
pub async fn purge(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let count: u8 = match args.single() {
        Ok(count) => count,
        Err(_) => {
            send_embed(ctx, msg, "Error", "You must provide a number of messages to purge (1-99).", 0xED4245).await?;
            return Ok(());
        }
    };

    if count == 0 || count > 99 {
        send_embed(ctx, msg, "Error", "Count must be between 1 and 99.", 0xED4245).await?;
        return Ok(());
    }

    let channel_id = msg.channel_id;
    let builder = GetMessages::new().limit(count + 1);
    let messages = channel_id.messages(&ctx.http, builder).await?;
    
    match channel_id.delete_messages(&ctx.http, messages).await {
        Ok(_) => {
            send_embed(ctx, msg, "Moderation: Purge", &format!("Successfully deleted {} messages.", count), 0x2b2d31).await?;
        },
        Err(e) => {
            send_embed(ctx, msg, "Error", &format!("Failed to delete messages: {}", e), 0xED4245).await?;
        }
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
#[required_permissions("MODERATE_MEMBERS")]
pub async fn timeout(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if msg.mentions.is_empty() {
        send_embed(ctx, msg, "Error", "You must mention a user to timeout.", 0xED4245).await?;
        return Ok(());
    }

    let user_to_timeout = &msg.mentions[0];
    args.advance();
    
    let duration_minutes: i64 = match args.single() {
        Ok(mins) => mins,
        Err(_) => {
            send_embed(ctx, msg, "Error", "You must provide a duration in minutes.", 0xED4245).await?;
            return Ok(());
        }
    };

    if duration_minutes <= 0 {
        send_embed(ctx, msg, "Error", "Duration must be greater than 0.", 0xED4245).await?;
        return Ok(());
    }

    let timestamp = Timestamp::from_unix_timestamp(Timestamp::now().unix_timestamp() + (duration_minutes * 60) as i64).unwrap();

    if let Some(guild_id) = msg.guild_id {
        let mut member = guild_id.member(&ctx.http, user_to_timeout.id).await?;
        match member.disable_communication_until_datetime(&ctx.http, timestamp).await {
            Ok(_) => {
                send_embed(ctx, msg, "Moderation: Timeout", &format!("User {} has been timed out for {} minutes.", user_to_timeout.name, duration_minutes), 0x2b2d31).await?;
            },
            Err(e) => {
                send_embed(ctx, msg, "Error", &format!("Failed to timeout user: {}", e), 0xED4245).await?;
            }
        }
    }

    Ok(())
}
