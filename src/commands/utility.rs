use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    builder::CreateMessage,
};
use crate::utils::embeds::{create_embed, send_embed};

#[command]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    send_embed(ctx, msg, "Pong", "System is online and responsive.", 0x2b2d31).await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn serverinfo(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = match ctx.http.get_guild_with_counts(guild_id).await {
        Ok(g) => g,
        Err(_) => {
            send_embed(ctx, msg, "Error", "Could not fetch server information.", 0xED4245).await?;
            return Ok(());
        }
    };

    let owner = guild.owner_id.to_user(&ctx.http).await.map(|u| u.name).unwrap_or_else(|_| "Unknown".to_string());
    let created_timestamp = guild_id.created_at().unix_timestamp();
    
    let desc = format!(
        "**General Information**\n\
        **Name:** {}\n\
        **ID:** {}\n\
        **Owner:** {} (`{}`)\n\
        **Created At:** <t:{}:F>\n\
        \n\
        **Statistics**\n\
        **Members:** {}\n\
        **Roles:** {}\n\
        **Boost Tier:** {:?}\n\
        **Boost Count:** {}",
        guild.name,
        guild.id,
        owner, guild.owner_id,
        created_timestamp,
        guild.approximate_member_count.unwrap_or(0),
        guild.roles.len(),
        guild.premium_tier,
        guild.premium_subscription_count.unwrap_or(0)
    );

    let mut embed = create_embed("Server Information", &desc, 0x2b2d31);
    
    if let Some(icon) = guild.icon_url() {
        embed = embed.thumbnail(icon);
    }
    
    if let Some(banner) = guild.banner_url() {
        embed = embed.image(banner);
    }

    let builder = CreateMessage::new().embed(embed);
    msg.channel_id.send_message(&ctx.http, builder).await?;

    Ok(())
}

#[command]
pub async fn userinfo(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let user = if msg.mentions.is_empty() {
        if let Ok(id) = args.single::<u64>() {
            match ctx.http.get_user(serenity::model::id::UserId::new(id)).await {
                Ok(u) => u,
                Err(_) => {
                    send_embed(ctx, msg, "Error", "User not found.", 0xED4245).await?;
                    return Ok(());
                }
            }
        } else {
            msg.author.clone()
        }
    } else {
        msg.mentions[0].clone()
    };

    let member = msg.guild_id.unwrap().member(&ctx.http, user.id).await.ok();
    let created_timestamp = user.id.created_at().unix_timestamp();
    
    let mut desc = format!(
        "**User Information**\n\
        **Username:** {}\n\
        **Global Name:** {}\n\
        **ID:** {}\n\
        **Bot:** {}\n\
        **Created At:** <t:{}:F>\n",
        user.name,
        user.global_name.as_deref().unwrap_or("None"),
        user.id,
        if user.bot { "Yes" } else { "No" },
        created_timestamp
    );

    if let Some(m) = member {
        if let Some(joined) = m.joined_at {
            let joined_ts = joined.unix_timestamp();
            desc.push_str(&format!("\n**Server Profile**\n**Joined At:** <t:{}:F>\n", joined_ts));
        }
        
        let roles = m.roles;
        if !roles.is_empty() {
            let roles_str: Vec<String> = roles.iter().map(|r| format!("<@&{}>", r)).collect();
            desc.push_str(&format!("\n**Roles ({})**\n{}", roles.len(), roles_str.join(", ")));
        }
    }

    let mut embed = create_embed("User Profile", &desc, 0x2b2d31);
    
    if let Some(avatar_url) = user.avatar_url() {
        embed = embed.thumbnail(avatar_url);
    }
    
    if let Some(banner) = user.banner_url() {
        embed = embed.image(banner);
    }

    let builder = CreateMessage::new().embed(embed);
    msg.channel_id.send_message(&ctx.http, builder).await?;

    Ok(())
}

#[command]
pub async fn avatar(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let user = if msg.mentions.is_empty() {
        if let Ok(id) = args.single::<u64>() {
            match ctx.http.get_user(serenity::model::id::UserId::new(id)).await {
                Ok(u) => u,
                Err(_) => {
                    send_embed(ctx, msg, "Error", "User not found.", 0xED4245).await?;
                    return Ok(());
                }
            }
        } else {
            msg.author.clone()
        }
    } else {
        msg.mentions[0].clone()
    };

    let url = user.face(); 
    let mut embed = create_embed(&format!("Avatar: {}", user.name), "", 0x2b2d31);
    embed = embed.image(url);
    
    let builder = CreateMessage::new().embed(embed);
    msg.channel_id.send_message(&ctx.http, builder).await?;
    
    Ok(())
}

#[command]
pub async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    let desc = "\
    **Music Commands**\n\
    `kh!join` - Joins the voice channel.\n\
    `kh!leave` - Leaves the voice channel.\n\
    `kh!play <url/query>` - Plays a song.\n\
    `kh!pause` - Pauses playback.\n\
    `kh!resume` - Resumes playback.\n\
    `kh!skip` - Skips the track.\n\
    `kh!stop` - Stops playback.\n\
    `kh!queue` - Shows queue length.\n\
    \n\
    **Moderation Commands**\n\
    `kh!kick <@user>` - Kicks a user.\n\
    `kh!ban <@user>` - Bans a user.\n\
    `kh!unban <User ID>` - Unbans a user.\n\
    `kh!purge <amount>` - Deletes messages.\n\
    `kh!timeout <@user> <minutes>` - Times out a user.\n\
    \n\
    **Admin Commands**\n\
    `kh!lock` - Locks the channel.\n\
    `kh!unlock` - Unlocks the channel.\n\
    `kh!slowmode <seconds>` - Sets slowmode.\n\
    \n\
    **Utility Commands**\n\
    `kh!help` - Displays this help message.\n\
    `kh!ping` - Checks bot latency.\n\
    `kh!serverinfo` - Server information.\n\
    `kh!userinfo [@user]` - User information.\n\
    `kh!avatar [@user]` - User avatar.\n\
    ";

    send_embed(ctx, msg, "Help - Command List", desc, 0x2b2d31).await?;
    Ok(())
}
