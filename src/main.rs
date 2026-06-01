use std::env;
use dotenvy::dotenv;
use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::{
        standard::{
            macros::{command, group},
            Args, CommandResult, StandardFramework,
        },
    },
    model::{channel::Message, gateway::Ready},
    prelude::GatewayIntents,
    builder::CreateMessage,
    builder::CreateEmbed,
};
use songbird::{
    input::{YoutubeDl, Compose},
    SerenityInit,
};
use tracing::{error, info};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Mousike (Rust) is connected as {}", ready.user.name);
    }
}

#[group]
#[commands(join, leave, play, pause, resume, skip, stop, queue)]
struct General;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv().ok(); // Reads .env if present

    let token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .group(&GENERAL_GROUP);

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}

#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let channel_id = guild.voice_states.get(&msg.author.id).and_then(|vs| vs.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            msg.channel_id.say(&ctx.http, "You must be in a voice channel first.").await?;
            return Ok(());
        }
    };

    let manager = songbird::get(ctx).await.expect("Songbird client placed at initialisation.").clone();
    let _ = manager.join(guild_id, connect_to).await;

    msg.channel_id.say(&ctx.http, "Joined voice channel.").await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases("dc", "disconnect")]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let manager = songbird::get(ctx).await.expect("Songbird client placed at initialisation.").clone();

    if manager.get(guild_id).is_some() {
        if let Err(e) = manager.remove(guild_id).await {
            msg.channel_id.say(&ctx.http, format!("Failed to leave: {:?}", e)).await?;
        } else {
            msg.channel_id.say(&ctx.http, "Left voice channel.").await?;
        }
    } else {
        msg.reply(ctx, "Not in a voice channel.").await?;
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases("p")]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            msg.channel_id.say(&ctx.http, "Must provide a URL or search query.").await?;
            return Ok(());
        }
    };

    let query = if !url.starts_with("http") {
        format!("ytsearch:{}", args.rest())
    } else {
        url
    };

    let guild_id = msg.guild_id.unwrap();
    let manager = songbird::get(ctx).await.expect("Songbird client.").clone();

    if manager.get(guild_id).is_none() {
        let guild = msg.guild(&ctx.cache).unwrap();
        let channel_id = guild.voice_states.get(&msg.author.id).and_then(|vs| vs.channel_id);

        if let Some(channel) = channel_id {
            manager.join(guild_id, channel).await.unwrap();
        } else {
            msg.channel_id.say(&ctx.http, "You must be in a voice channel to play music.").await?;
            return Ok(());
        }
    }

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let http_client = reqwest::Client::new();
        let src = YoutubeDl::new(http_client, query);
        
        let _ = handler.enqueue_input(src.into()).await;
        
        msg.channel_id.say(&ctx.http, "Added to queue!").await?;
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn pause(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let manager = songbird::get(ctx).await.expect("Songbird Voice client placed in at initialisation.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        let _ = queue.pause();
        msg.channel_id.say(&ctx.http, "Paused playback.").await?;
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn resume(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let manager = songbird::get(ctx).await.expect("Songbird Voice client placed in at initialisation.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        let _ = queue.resume();
        msg.channel_id.say(&ctx.http, "Resumed playback.").await?;
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases("s")]
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let manager = songbird::get(ctx).await.expect("Songbird Voice client placed in at initialisation.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        let _ = queue.skip();
        msg.channel_id.say(&ctx.http, "Skipped the current track.").await?;
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let manager = songbird::get(ctx).await.expect("Songbird Voice client placed in at initialisation.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        queue.stop();
        msg.channel_id.say(&ctx.http, "Stopped and cleared queue.").await?;
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases("q")]
async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let manager = songbird::get(ctx).await.expect("Songbird Voice client placed in at initialisation.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue().current_queue();
        let len = queue.len();
        msg.channel_id.say(&ctx.http, format!("There are {} tracks in the queue.", len)).await?;
    }
    Ok(())
}
