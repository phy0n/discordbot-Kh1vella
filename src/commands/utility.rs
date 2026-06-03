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
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;
    let name = &guild.name;
    let member_count = guild.approximate_member_count.unwrap_or(0);

    let desc = format!("**Name:** {}\n**ID:** {}\n**Members:** {}", name, guild_id, member_count);
    
    send_embed(ctx, msg, "Server Information", &desc, 0x2b2d31).await?;
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

    let desc = format!("**Username:** {}\n**ID:** {}", user.name, user.id);
    let mut embed = create_embed("User Information", &desc, 0x2b2d31);
    
    if let Some(avatar_url) = user.avatar_url() {
        embed = embed.thumbnail(avatar_url);
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
    `kh!leave` / `kh!dc` - Leaves the voice channel.\n\
    `kh!play <url/query>` / `kh!p` - Plays a song.\n\
    `kh!pause` - Pauses playback.\n\
    `kh!resume` - Resumes playback.\n\
    `kh!skip` / `kh!s` - Skips the track.\n\
    `kh!stop` - Stops playback.\n\
    `kh!queue` / `kh!q` - Shows queue length.\n\
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
