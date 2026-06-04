#![allow(deprecated)]

mod commands;
mod utils;
mod handler;
mod types;
mod api;
pub mod db;
pub mod services;

use std::env;
use dotenvy::dotenv;
use serenity::prelude::GatewayIntents;
use songbird::SerenityInit;
use tracing::{error, info};
use sqlx::postgres::PgPoolOptions;

use crate::types::Data;
use crate::commands::{
    music::*,
    moderation::*,
    utility::*,
    admin::*,
};

#[tokio::main]
async fn main() {
    let start_time = std::time::Instant::now();
    tracing_subscriber::fmt::init();
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment");
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| env::var("SUPABASE_DATABASE_URL").expect("Expected DATABASE_URL"));

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");
    info!("Connected to Supabase PostgreSQL");

    let intents = GatewayIntents::non_privileged() 
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_MESSAGES;

    let chatbot_state = std::sync::Arc::new(tokio::sync::RwLock::new(true));
    let api_chatbot_state = chatbot_state.clone();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                join(), leave(), play(), pause(), resume(), skip(), stop(), queue(),
                kick(), ban(), unban(), purge(), timeout(), warn(), strike(),
                lock(), unlock(), slowmode(), chatbot(),
                ping(), userinfo(), serverinfo(), avatar(), help(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("kh!".into()),
                ..Default::default()
            },
            event_handler: |ctx, event, framework, data| {
                Box::pin(handler::event_handler(ctx, event, framework, data))
            },
            owners: std::collections::HashSet::from([serenity::model::id::UserId::new(494169184175915019)]),
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    chatbot_enabled: chatbot_state,
                    db_pool: pool,
                })
            })
        })
        .build();

    let mut client = serenity::Client::builder(&token, intents)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Error creating client");

    let cache = client.cache.clone();
    let http = client.http.clone();
    tokio::spawn(async move {
        api::start_api_server(api_chatbot_state, cache, http, start_time).await;
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
