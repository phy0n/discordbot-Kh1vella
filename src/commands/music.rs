use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};
use songbird::input::YoutubeDl;
use crate::utils::embeds::send_embed;

#[command]
#[only_in(guilds)]
pub async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let channel_id = {
        let guild = msg.guild(&ctx.cache).unwrap();
        guild.voice_states.get(&msg.author.id).and_then(|vs| vs.channel_id)
    };

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            send_embed(ctx, msg, "Action Required", "You must be in a voice channel to use this command.", 0x2b2d31).await?;
            return Ok(());
        }
    };

    let manager = songbird::get(ctx).await.expect("Songbird client.").clone();
    let _ = manager.join(guild_id, connect_to).await;

    send_embed(ctx, msg, "Voice Channel", "Successfully connected to the voice channel.", 0x2b2d31).await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases("dc", "disconnect")]
pub async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let manager = songbird::get(ctx).await.expect("Songbird client.").clone();

    if manager.get(guild_id).is_some() {
        if let Err(e) = manager.remove(guild_id).await {
            send_embed(ctx, msg, "Error", &format!("Failed to disconnect: {:?}", e), 0xED4245).await?;
        } else {
            send_embed(ctx, msg, "Voice Channel", "Successfully disconnected from the voice channel.", 0x2b2d31).await?;
        }
    } else {
        send_embed(ctx, msg, "Status", "Currently not connected to any voice channel.", 0x2b2d31).await?;
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases("p")]
pub async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            send_embed(ctx, msg, "Input Required", "Please provide a valid URL or search query.", 0xED4245).await?;
            return Ok(());
        }
    };

    let query = if !url.starts_with("http") {
        let rest = args.rest();
        let full_query = if rest.is_empty() {
            url
        } else {
            format!("{} {}", url, rest)
        };
        format!("ytsearch:{}", full_query)
    } else {
        url
    };

    let guild_id = msg.guild_id.unwrap();
    let manager = songbird::get(ctx).await.expect("Songbird client.").clone();

    if manager.get(guild_id).is_none() {
        let channel_id = {
            let guild = msg.guild(&ctx.cache).unwrap();
            guild.voice_states.get(&msg.author.id).and_then(|vs| vs.channel_id)
        };

        if let Some(channel) = channel_id {
            manager.join(guild_id, channel).await.unwrap();
        } else {
            send_embed(ctx, msg, "Action Required", "You must be in a voice channel to play music.", 0x2b2d31).await?;
            return Ok(());
        }
    }

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        let http_client = reqwest::Client::new();
        let src = YoutubeDl::new(http_client, query);
        
        let _ = handler.enqueue_input(src.into()).await;
        send_embed(ctx, msg, "Playback", "Track has been added to the queue.", 0x2b2d31).await?;
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn pause(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let manager = songbird::get(ctx).await.expect("Songbird client.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let _ = handler.queue().pause();
        send_embed(ctx, msg, "Playback", "Playback paused.", 0x2b2d31).await?;
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn resume(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let manager = songbird::get(ctx).await.expect("Songbird client.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let _ = handler.queue().resume();
        send_embed(ctx, msg, "Playback", "Playback resumed.", 0x2b2d31).await?;
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases("s")]
pub async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let manager = songbird::get(ctx).await.expect("Songbird client.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let _ = handler.queue().skip();
        send_embed(ctx, msg, "Playback", "Track skipped.", 0x2b2d31).await?;
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let manager = songbird::get(ctx).await.expect("Songbird client.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        handler.queue().stop();
        send_embed(ctx, msg, "Playback", "Playback stopped and queue cleared.", 0x2b2d31).await?;
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases("q")]
pub async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let manager = songbird::get(ctx).await.expect("Songbird client.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue().current_queue();
        let len = queue.len();
        send_embed(ctx, msg, "Queue", &format!("There are {} tracks currently in the queue.", len), 0x2b2d31).await?;
    }
    Ok(())
}
