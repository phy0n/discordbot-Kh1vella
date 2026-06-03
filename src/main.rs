#![allow(deprecated)]

mod commands;
mod utils;
mod handler;

use std::env;
use std::collections::HashSet;
use dotenvy::dotenv;
use serenity::{
    client::Client,
    framework::StandardFramework,
    framework::standard::macros::group,
    model::id::UserId,
    prelude::GatewayIntents,
};
use songbird::SerenityInit;
use tracing::error;

use crate::handler::Handler;
use crate::commands::{
    music::*,
    moderation::*,
    utility::*,
    admin::*,
};

const OWNER_ID: u64 = 494169184175915019;

#[group]
#[commands(
    join, leave, play, pause, resume, skip, stop, queue,
    ping, userinfo, serverinfo, avatar, help
)]
struct Public;

#[group]
#[owners_only]
#[commands(
    kick, ban, unban, purge, timeout,
    lock, unlock, slowmode, chatbot
)]
struct Owner;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment");

    let mut owners = HashSet::new();
    owners.insert(UserId::new(OWNER_ID));

    let config = serenity::framework::standard::Configuration::new()
        .prefix("kh!")
        .owners(owners);

    let mut framework = StandardFramework::new()
        .group(&PUBLIC_GROUP)
        .group(&OWNER_GROUP);
    framework.configure(config);

    let intents = GatewayIntents::non_privileged() 
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_MESSAGES;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .type_map_insert::<crate::handler::ChatbotState>(std::sync::Arc::new(tokio::sync::RwLock::new(true)))
        .register_songbird()
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
