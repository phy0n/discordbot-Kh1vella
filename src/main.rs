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

    let pool = match PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("CRITICAL ERROR: Failed to connect to database! URL: {}", database_url);
            eprintln!("Error details: {:?}", e);
            std::process::exit(1);
        }
    };
    info!("Connected to Supabase PostgreSQL");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS khivella_memory (
            user_id TEXT PRIMARY KEY,
            username TEXT NOT NULL,
            favorite_game TEXT,
            favorite_food TEXT,
            about_user TEXT,
            relationship_score INT DEFAULT 0,
            last_interaction TIMESTAMPTZ DEFAULT NOW()
        );"
    ).execute(&pool).await.unwrap_or_else(|e| {
        error!("Failed to initialize khivella_memory table: {:?}", e);
        Default::default()
    });

    let intents = GatewayIntents::non_privileged() 
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_MESSAGES;

    let chatbot_state = std::sync::Arc::new(tokio::sync::RwLock::new(true));
    let chat_history = std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new()));
    let api_chatbot_state = chatbot_state.clone();

    let framework_pool = pool.clone();
    let api_pool = pool.clone();

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
                    db_pool: framework_pool,
                    chat_history,
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
    let api_task = tokio::spawn(async move {
        api::start_api_server(api_chatbot_state, cache, http, start_time, api_pool).await;
    });

    if let Err(why) = client.start().await {
        error!("CRITICAL ERROR: Bot failed to connect to Discord! Error: {:?}", why);
        eprintln!("CRITICAL ERROR: Bot failed to connect to Discord! Error: {:?}", why);
        eprintln!("Keeping the process alive so the API server can still respond...");
        let _ = api_task.await;
    }
}
