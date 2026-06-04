use serde::{Deserialize, Serialize};
use serde_json::json;
use serenity::all::Message;
use serenity::client::Context as SerenityContext;
use std::env;
use tracing::{error, info};
use crate::types::Data;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct GeminiResponse {
    reply: String,
    memory_updates: Option<MemoryUpdates>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct MemoryUpdates {
    favorite_game: Option<String>,
    favorite_food: Option<String>,
    about_user: Option<String>,
}

#[derive(sqlx::FromRow, Default)]
struct UserMemory {
    user_id: String,
    username: String,
    favorite_game: Option<String>,
    favorite_food: Option<String>,
    about_user: Option<String>,
    relationship_score: i32,
}

fn get_local_response(prompt: &str) -> Option<&'static str> {
    let lower = prompt.to_lowercase();
    let lower_trim = lower.trim();
    
    match lower_trim {
        "pagi" | "selamat pagi" | "met pagi" => Some("Pagi juga!"),
        "malam" | "selamat malam" | "met malem" => Some("Malam!"),
        "siang" => Some("Siang!"),
        "halo" | "haloo" | "hi" | "hai" | "oi" | "weh" => Some("Halo!"),
        "wkwk" | "wkwkwk" | "awokwok" | "haha" => Some("wkwk"),
        "test" | "tes" => Some("masuk jir"),
        "bjir" | "njir" | "anjir" | "jir" => Some("jir"),
        "gatau" | "g" => Some("yaudah"),
        "ok" | "oke" | "y" => Some("ok"),
        "brb" | "afk" => Some("oke tiati"),
        _ => None,
    }
}

pub async fn handle_chat(ctx: &SerenityContext, msg: &Message, data: &Data, prompt: &str) {
    let user_id = msg.author.id.get();
    let username = msg.author.name.clone();
    if let Some(local_reply) = get_local_response(prompt) {
        let _ = msg.reply(&ctx.http, local_reply).await;
        let _ = sqlx::query("
            INSERT INTO khivella_memory (user_id, username, relationship_score, last_interaction)
            VALUES ($1, $2, 1, NOW())
            ON CONFLICT (user_id) DO UPDATE 
            SET relationship_score = LEAST(khivella_memory.relationship_score + 1, 100),
                last_interaction = NOW()
        ")
        .bind(user_id.to_string())
        .bind(&username)
        .execute(&data.db_pool)
        .await;

        return;
    }

    let mut memory: UserMemory = sqlx::query_as("SELECT * FROM khivella_memory WHERE user_id = $1")
        .bind(user_id.to_string())
        .fetch_optional(&data.db_pool)
        .await
        .unwrap_or(None)
        .unwrap_or_default();

    let relationship_context = if user_id == 494169184175915019 {
        "USER INI ADALAH PHION! Dia adalah cowok sahabat masa kecilmu dan kamu diam-diam naksir/suka sama dia. Bicaralah dengannya dengan sangat akrab, agak tsundere, gengsi, kadang salting (salah tingkah), tapi sebenarnya kamu sangat peduli dan nyaman ngobrol berdua sama dia. Ingat, kamu gengsi buat ngaku kalau kamu suka dia."
    } else if memory.relationship_score < 20 {
        "User ini orang asing / baru kenal. Bicaralah sedikit lebih kaku atau jutek."
    } else if memory.relationship_score < 40 {
        "User ini sekadar kenalan (acquaintance). Lumayan biasa saja."
    } else if memory.relationship_score < 60 {
        "User ini teman (friend). Kamu sudah lumayan akrab dan santai."
    } else if memory.relationship_score < 80 {
        "User ini teman dekat (close friend). Bisa saling bercanda bebas dan kadang nge-roast."
    } else {
        "User ini sahabat (best friend). Bicaralah sangat natural, bebas, dan sangat akrab."
    };

    let base_prompt = std::fs::read_to_string("khivella_prompt.txt")
        .unwrap_or_else(|_| "Kamu adalah Khivella, cewek SMA 17 tahun di Surabaya yang santai.".to_string());
        
    let jakarta_offset = chrono::FixedOffset::east_opt(7 * 3600).unwrap();
    let current_time = chrono::Utc::now().with_timezone(&jakarta_offset).format("%Y-%m-%d %H:%M WIB").to_string();

    let system_instruction = format!(r#"{}
    
    [WAKTU SAAT INI]
    Waktu: {}
    
    [MEMORI USER: {}]
    - Relationship Score: {}/100 ({})
    - Game Favorit: {}
    - Makanan Favorit: {}
    - Fakta Lain: {}
    
    [INSTRUKSI OUTPUT (WAJIB JSON)]
    Balas dalam format JSON yang valid dan di-parse:
    {{
        "reply": "Balasan chatmu ke user",
        "memory_updates": {{
            "favorite_game": "isi jika user menyebutkan game favoritnya, jika tidak null",
            "favorite_food": "isi jika user menyebutkan makanan favoritnya, jika tidak null",
            "about_user": "isi fakta penting lainnya tentang user, jika tidak null"
        }}
    }}
    "#, 
        base_prompt, 
        current_time, 
        username,
        memory.relationship_score,
        relationship_context,
        memory.favorite_game.as_deref().unwrap_or("Belum diketahui"),
        memory.favorite_food.as_deref().unwrap_or("Belum diketahui"),
        memory.about_user.as_deref().unwrap_or("Belum diketahui")
    );

    let mut current_history = {
        let hist_lock = data.chat_history.read().await;
        hist_lock.get(&user_id).cloned().unwrap_or_default()
    };

    current_history.push(json!({
        "role": "user",
        "parts": [{"text": prompt}]
    }));

    let api_key = env::var("GEMINI_API_KEY").unwrap_or_default();
    let model = env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-1.5-flash".to_string());
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}", model, api_key);
    
    let client = reqwest::Client::new();
    let body = json!({
        "systemInstruction": {
            "parts": [{"text": system_instruction}]
        },
        "contents": current_history,
        "generationConfig": {
            "responseMimeType": "application/json"
        }
    });

    match client.post(&url).json(&body).send().await {
        Ok(res) => {
            if let Ok(json_res) = res.json::<serde_json::Value>().await {
                if let Some(text) = json_res["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                    match serde_json::from_str::<GeminiResponse>(text) {
                        Ok(gemini_data) => {
                            let _ = msg.reply(&ctx.http, &gemini_data.reply).await;

                            current_history.push(json!({
                                "role": "model",
                                "parts": [{"text": gemini_data.reply}]
                            }));

                            if current_history.len() > 6 {
                                let start = current_history.len() - 6;
                                current_history = current_history[start..].to_vec();
                            }
                            
                            let mut hist_lock = data.chat_history.write().await;
                            hist_lock.insert(user_id, current_history);

                            let new_score = std::cmp::min(memory.relationship_score + 1, 100);
                            let fav_game = gemini_data.memory_updates.as_ref().and_then(|m| m.favorite_game.clone()).or(memory.favorite_game);
                            let fav_food = gemini_data.memory_updates.as_ref().and_then(|m| m.favorite_food.clone()).or(memory.favorite_food);
                            let about_u = gemini_data.memory_updates.as_ref().and_then(|m| m.about_user.clone()).or(memory.about_user);

                            let _ = sqlx::query("
                                INSERT INTO khivella_memory (user_id, username, favorite_game, favorite_food, about_user, relationship_score, last_interaction)
                                VALUES ($1, $2, $3, $4, $5, $6, NOW())
                                ON CONFLICT (user_id) DO UPDATE 
                                SET username = $2, favorite_game = $3, favorite_food = $4, about_user = $5, relationship_score = $6, last_interaction = NOW()
                            ")
                            .bind(user_id.to_string())
                            .bind(&username)
                            .bind(fav_game)
                            .bind(fav_food)
                            .bind(about_u)
                            .bind(new_score)
                            .execute(&data.db_pool)
                            .await;
                        }
                        Err(e) => {
                            error!("Failed to parse JSON from Gemini: {}", e);
                            let _ = msg.reply(&ctx.http, "Eh sorry gw error dikit, gatau mau jawab apa (JSON parse error).").await;
                        }
                    }
                } else {
                    let _ = msg.reply(&ctx.http, "Eh sori error limit kayaknya.").await;
                }
            }
        }
        Err(e) => {
            error!("Request failed: {}", e);
            let _ = msg.reply(&ctx.http, "Koneksi ke AI lagi jelek nih, sorry ya.").await;
        }
    }
}
