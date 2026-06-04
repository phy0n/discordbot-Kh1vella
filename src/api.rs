use axum::{
    routing::{get, post},
    Router,
    extract::State,
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use tokio::sync::RwLock;
use serenity::all::CacheAndHttp;
use serenity::model::id::ChannelId;

#[derive(Serialize)]
struct GuildInfo {
    id: String,
    name: String,
    member_count: u64,
}

#[derive(Serialize)]
struct StatusResponse {
    status: String,
    chatbot_enabled: bool,
    guilds: Vec<GuildInfo>,
    total_members: u64,
}

#[derive(Clone)]
pub struct ApiState {
    pub chatbot_enabled: Arc<RwLock<bool>>,
    pub discord: Arc<CacheAndHttp>,
}

async fn get_status(State(state): State<ApiState>) -> Json<StatusResponse> {
    let enabled = *state.chatbot_enabled.read().await;
    
    let mut guilds_info = Vec::new();
    let mut total_members = 0;
    
    let guild_ids = state.discord.cache.guilds();
    for guild_id in guild_ids {
        if let Some(guild) = state.discord.cache.guild(guild_id) {
            let count = guild.member_count as u64;
            guilds_info.push(GuildInfo {
                id: guild.id.to_string(),
                name: guild.name.clone(),
                member_count: count,
            });
            total_members += count;
        }
    }

    Json(StatusResponse {
        status: "online".to_string(),
        chatbot_enabled: enabled,
        guilds: guilds_info,
        total_members,
    })
}

#[derive(Deserialize)]
struct ToggleRequest {
    enabled: bool,
}

async fn toggle_chatbot(
    State(state): State<ApiState>,
    Json(payload): Json<ToggleRequest>,
) -> Json<StatusResponse> {
    *state.chatbot_enabled.write().await = payload.enabled;
    get_status(State(state)).await
}

#[derive(Deserialize)]
struct SendMessageRequest {
    channel_id: String,
    content: String,
}

#[derive(Serialize)]
struct SendMessageResponse {
    success: bool,
    error: Option<String>,
}

async fn send_message(
    State(state): State<ApiState>,
    Json(payload): Json<SendMessageRequest>,
) -> Json<SendMessageResponse> {
    if let Ok(ch_id) = payload.channel_id.parse::<u64>() {
        let channel_id = ChannelId::new(ch_id);
        let builder = serenity::builder::CreateMessage::new().content(payload.content);
        
        match channel_id.send_message(&state.discord.http, builder).await {
            Ok(_) => Json(SendMessageResponse { success: true, error: None }),
            Err(e) => Json(SendMessageResponse { success: false, error: Some(e.to_string()) }),
        }
    } else {
        Json(SendMessageResponse { success: false, error: Some("Invalid channel ID format".to_string()) })
    }
}

pub async fn start_api_server(chatbot_state: Arc<RwLock<bool>>, discord: Arc<CacheAndHttp>) {
    let api_state = ApiState {
        chatbot_enabled: chatbot_state,
        discord,
    };

    let app = Router::new()
        .route("/api/status", get(get_status))
        .route("/api/chatbot/toggle", post(toggle_chatbot))
        .route("/api/message/send", post(send_message))
        .layer(CorsLayer::permissive())
        .with_state(api_state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("API Server running on {}", addr);
    
    axum::serve(listener, app).await.unwrap();
}
