use serde_json::json;
use std::env;

pub async fn ask_gemini(prompt: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let api_key = env::var("GEMINI_API_KEY")?;
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}", api_key);
    
    let client = reqwest::Client::new();
    let body = json!({
        "contents": [{
            "parts": [{"text": prompt}]
        }]
    });
    
    let res = client.post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;
        
    let json_res: serde_json::Value = res.json().await?;
    
    if let Some(text) = json_res["candidates"][0]["content"]["parts"][0]["text"].as_str() {
        Ok(text.to_string())
    } else {
        if let Some(err_msg) = json_res["error"]["message"].as_str() {
            Err(err_msg.into())
        } else {
            Err("Failed to parse Gemini API response".into())
        }
    }
}
