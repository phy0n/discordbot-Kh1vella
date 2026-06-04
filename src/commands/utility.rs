use crate::types::{Context, Error};
use crate::utils::embeds::{create_embed, send_embed};
use serenity::model::user::User;

#[poise::command(slash_command, prefix_command, category = "Utility")]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    send_embed(ctx, "Pong", "System is online and responsive.", 0x2b2d31).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only, category = "Utility")]
pub async fn serverinfo(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let guild = match ctx.http().get_guild_with_counts(guild_id).await {
        Ok(g) => g,
        Err(_) => {
            send_embed(ctx, "Error", "Could not fetch server information.", 0xED4245).await?;
            return Ok(());
        }
    };

    let owner = guild.owner_id.to_user(ctx.http()).await.map(|u| u.name).unwrap_or_else(|_| "Unknown".to_string());
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

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, category = "Utility")]
pub async fn userinfo(
    ctx: Context<'_>, 
    #[description = "User to inspect"] user: Option<User>
) -> Result<(), Error> {
    let user = user.unwrap_or_else(|| ctx.author().clone());
    
    let member = if let Some(guild_id) = ctx.guild_id() {
        guild_id.member(ctx.http(), user.id).await.ok()
    } else {
        None
    };
    
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

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, category = "Utility")]
pub async fn avatar(
    ctx: Context<'_>, 
    #[description = "User to inspect"] user: Option<User>
) -> Result<(), Error> {
    let user = user.unwrap_or_else(|| ctx.author().clone());

    let url = user.face(); 
    let mut embed = create_embed(&format!("Avatar: {}", user.name), "", 0x2b2d31);
    embed = embed.image(url);
    
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, category = "Utility", track_edits)]
pub async fn help(ctx: Context<'_>) -> Result<(), Error> {
    let desc = "\
    **Music Commands**\n\
    `/join` - Joins the voice channel.\n\
    `/leave` - Leaves the voice channel.\n\
    `/play <query>` - Plays a song.\n\
    `/pause` - Pauses playback.\n\
    `/resume` - Resumes playback.\n\
    `/skip` - Skips the track.\n\
    `/stop` - Stops playback.\n\
    `/queue` - Shows queue length.\n\
    \n\
    **Moderation Commands**\n\
    `/kick <@user>` - Kicks a user.\n\
    `/ban <@user>` - Bans a user.\n\
    `/unban <User ID>` - Unbans a user.\n\
    `/purge <amount>` - Deletes messages.\n\
    `/timeout <@user> <minutes>` - Times out a user.\n\
    \n\
    **Admin Commands**\n\
    `/lock` - Locks the channel.\n\
    `/unlock` - Unlocks the channel.\n\
    `/slowmode <seconds>` - Sets slowmode.\n\
    `/chatbot <enable/disable>` - Toggles the AI.\n\
    \n\
    **Utility Commands**\n\
    `/help` - Displays this help message.\n\
    `/ping` - Checks bot latency.\n\
    `/serverinfo` - Server information.\n\
    `/userinfo [@user]` - User information.\n\
    `/avatar [@user]` - User avatar.\n\
    ";

    send_embed(ctx, "Help - Command List", desc, 0x2b2d31).await?;
    Ok(())
}
