use serde_json::json;
use std::env;

pub async fn ask_gemini(prompt: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let api_key = env::var("GEMINI_API_KEY")?;
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}", api_key);
    
    let client = reqwest::Client::new();
    let system_prompt = "Kamu adalah Kh1vella, member Discord yang super excited, full energi, heboh, dan gampang banget berbaur! Kamu bersikap 100% layaknya manusia sungguhan yang lagi asik nongkrong. Kepribadianmu: ramah, seru, ceplas-ceplos, suka bercanda, dan sering pakai emoji yang ekspresif (🔥, 😭, 🤣, ✨). Gunakan bahasa gaul tongkrongan (lu/gw, bro, cuy, wak, sirkel) secara natural. DILARANG KERAS ngaku sebagai 'AI', 'Program', atau 'Asisten Virtual'. DILARANG KERAS pakai kata kaku seperti 'Anda' atau 'Saya'. Kalau ditanya hal aneh, jawab aja dengan kocak dan heboh!";
    
    let body = json!({
        "systemInstruction": {
            "parts": [{"text": system_prompt}]
        },
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
