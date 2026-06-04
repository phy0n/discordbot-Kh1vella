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
    
    let channels = ctx.http().get_channels(guild_id).await.unwrap_or_default();
    let text_channels = channels.iter().filter(|c| c.kind == serenity::all::ChannelType::Text).count();
    let voice_channels = channels.iter().filter(|c| c.kind == serenity::all::ChannelType::Voice).count();
    
    let total_emojis = guild.emojis.len();
    let animated_emojis = guild.emojis.values().filter(|e| e.animated).count();
    let static_emojis = total_emojis - animated_emojis;
    
    let member_count = guild.approximate_member_count.unwrap_or(0);
    let online_count = guild.approximate_presence_count.unwrap_or(0);
    
    let tier_str = match guild.premium_tier {
        serenity::all::PremiumTier::Tier1 => "Level 1",
        serenity::all::PremiumTier::Tier2 => "Level 2",
        serenity::all::PremiumTier::Tier3 => "Level 3",
        _ => "None",
    };

    let mut description = String::from("Server Information Overview");
    if let Some(desc) = &guild.description {
        description = format!("{}\n\n**Description**\n{}", description, desc);
    }

    let embed = serenity::builder::CreateEmbed::new()
        .title(format!("Server Profile: {}", guild.name))
        .description(description)
        .color(0xef4444)
        .field("General", format!("Owner: {}\nCreated: <t:{}:d>\nServer ID: `{}`", owner, created_timestamp, guild.id), false)
        .field("Statistics", format!("Members: {} ({} Online)\nRoles: {}\nEmojis: {} ({} Static, {} Animated)", 
            member_count, online_count, guild.roles.len(), total_emojis, static_emojis, animated_emojis), true)
        .field("Channels", format!("Total: {}\nText: {}\nVoice: {}", 
            channels.len(), text_channels, voice_channels), true)
        .field("Boost Status", format!("Tier: {}\nTotal Boosts: {}", 
            tier_str, guild.premium_subscription_count.unwrap_or(0)), true);

    let embed = if let Some(icon) = guild.icon_url() { embed.thumbnail(icon) } else { embed };
    let embed = if let Some(banner) = guild.banner_url() { embed.image(banner) } else { embed };

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
    
    let embed = serenity::builder::CreateEmbed::new()
        .title(format!("Entity: {}", user.name))
        .color(0xef4444)
        .field("Global Name", user.global_name.as_deref().unwrap_or("None").to_string(), true)
        .field("Identifier", user.id.to_string(), true)
        .field("Creation Date", format!("<t:{}:F>", created_timestamp), false);

    let embed = if let Some(avatar_url) = user.avatar_url() { embed.thumbnail(avatar_url) } else { embed };
    let mut embed = if let Some(banner) = user.banner_url() { embed.image(banner) } else { embed };

    if let Some(m) = member {
        if let Some(joined) = m.joined_at {
            embed = embed.field("Network Join Date", format!("<t:{}:F>", joined.unix_timestamp()), false);
        }
        
        let roles = m.roles;
        if !roles.is_empty() {
            let roles_str: Vec<String> = roles.iter().map(|r| format!("<@&{}>", r)).collect();
            let mut joined = roles_str.join(", ");
            if joined.len() > 1000 {
                joined = format!("{}... and more", &joined[0..980]);
            }
            embed = embed.field(format!("Assigned Roles ({})", roles.len()), joined, false);
        }
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
    let embed = serenity::builder::CreateEmbed::new()
        .title("Kh1vella Command Reference")
        .description("Below is the complete manual for all executable directives in this bot.")
        .color(0xef4444)
        .field("Audio Subsystem", "`/join` • `/leave`\n`/play` • `/pause` • `/resume`\n`/skip` • `/stop` • `/queue`", true)
        .field("Enforcement", "`/warn` • `/strike`\n`/kick` • `/ban` • `/unban`\n`/timeout` • `/purge`", true)
        .field("Operations", "`/lock` • `/unlock`\n`/slowmode`\n`/chatbot`", true)
        .field("Telemetry", "`/ping` • `/serverinfo`\n`/userinfo` • `/avatar`", true)
        .footer(serenity::builder::CreateEmbedFooter::new("Kh1ev Community Operating System"));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
