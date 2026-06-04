use crate::types::{Context, Error};
use crate::utils::embeds::send_embed;
use songbird::input::YoutubeDl;
use serenity::all::Mentionable;

#[poise::command(slash_command, prefix_command, guild_only, category = "Music")]
pub async fn join(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let channel_id = {
        let guild = ctx.guild().unwrap();
        guild.voice_states.get(&ctx.author().id)
            .and_then(|voice_state| voice_state.channel_id)
    };

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            send_embed(ctx, "Error", "You need to join a voice channel first.", 0xED4245).await?;
            return Ok(());
        }
    };

    let manager = songbird::get(ctx.serenity_context()).await.unwrap().clone();
    
    if let Ok(_handler_lock) = manager.join(guild_id, connect_to).await {
        send_embed(ctx, "Voice Channel", &format!("Successfully connected to {}.", connect_to.mention()), 0x2b2d31).await?;
    } else {
        send_embed(ctx, "Error", "Failed to join the voice channel.", 0xED4245).await?;
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only, category = "Music")]
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let manager = songbird::get(ctx.serenity_context()).await.unwrap().clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            send_embed(ctx, "Error", &format!("Failed to leave the voice channel: {:?}", e), 0xED4245).await?;
        } else {
            send_embed(ctx, "Voice Channel", "Disconnected from the voice channel.", 0x2b2d31).await?;
        }
    } else {
        send_embed(ctx, "Error", "I am not in a voice channel.", 0xED4245).await?;
    }

    Ok(())
}

use songbird::events::{Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent};
use serenity::async_trait;

struct TrackErrorNotifier;

#[async_trait]
impl VoiceEventHandler for TrackErrorNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            for (state, _handle) in *track_list {
                tracing::error!("Track encountered an error during playback! State: {:?}", state.playing);
            }
        }
        None
    }
}

#[poise::command(slash_command, prefix_command, guild_only, category = "Music")]
pub async fn play(
    ctx: Context<'_>, 
    #[description = "Search query or URL"] 
    #[rest] query: String
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let manager = songbird::get(ctx.serenity_context()).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let http_client = reqwest::Client::new();
        let query_text = if query.starts_with("http") {
            query
        } else {
            format!("ytsearch:{}", query)
        };
        
        let src = YoutubeDl::new(http_client, query_text);
        let track_handle = handler.enqueue_input(src.into()).await;
        
        let _ = track_handle.add_event(Event::Track(TrackEvent::Error), TrackErrorNotifier);
        
        send_embed(ctx, "Playback", "Track has been added to the queue.", 0x2b2d31).await?;
    } else {
        send_embed(ctx, "Error", "I am not in a voice channel. Use `/join` first.", 0xED4245).await?;
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only, category = "Music")]
pub async fn pause(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let manager = songbird::get(ctx.serenity_context()).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        
        if queue.is_empty() {
            send_embed(ctx, "Playback", "Queue is empty.", 0x2b2d31).await?;
            return Ok(());
        }

        let _ = queue.pause();
        send_embed(ctx, "Playback", "Paused the current track.", 0x2b2d31).await?;
    } else {
        send_embed(ctx, "Error", "I am not in a voice channel.", 0xED4245).await?;
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only, category = "Music")]
pub async fn resume(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let manager = songbird::get(ctx.serenity_context()).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        
        if queue.is_empty() {
            send_embed(ctx, "Playback", "Queue is empty.", 0x2b2d31).await?;
            return Ok(());
        }

        let _ = queue.resume();
        send_embed(ctx, "Playback", "Resumed the current track.", 0x2b2d31).await?;
    } else {
        send_embed(ctx, "Error", "I am not in a voice channel.", 0xED4245).await?;
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only, category = "Music")]
pub async fn skip(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let manager = songbird::get(ctx.serenity_context()).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        
        if queue.is_empty() {
            send_embed(ctx, "Playback", "Queue is empty.", 0x2b2d31).await?;
            return Ok(());
        }

        let _ = queue.skip();
        send_embed(ctx, "Playback", "Skipped the current track.", 0x2b2d31).await?;
    } else {
        send_embed(ctx, "Error", "I am not in a voice channel.", 0xED4245).await?;
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only, category = "Music")]
pub async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let manager = songbird::get(ctx.serenity_context()).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        
        queue.stop();
        send_embed(ctx, "Playback", "Stopped playing and cleared the queue.", 0x2b2d31).await?;
    } else {
        send_embed(ctx, "Error", "I am not in a voice channel.", 0xED4245).await?;
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only, category = "Music")]
pub async fn queue(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let manager = songbird::get(ctx.serenity_context()).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        let count = queue.len();
        
        send_embed(ctx, "Queue", &format!("There are currently {} track(s) in the queue.", count), 0x2b2d31).await?;
    } else {
        send_embed(ctx, "Error", "I am not in a voice channel.", 0xED4245).await?;
    }

    Ok(())
}
