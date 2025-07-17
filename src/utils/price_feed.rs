use reqwest::Client;
use serde::Deserialize;
use std::env;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use crate::utils::telegram::TelegramBot;

use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref POOL_CACHE: Mutex<HashMap<String, PoolInfo>> = Mutex::new(HashMap::new());
}

#[derive(Deserialize)]
struct JupiterQuote {
    data: Vec<QuoteData>,
}

#[derive(Deserialize)]
struct QuoteData {
    out_amount: f64,
}

#[derive(Deserialize)]
struct RaydiumPool {
    id: String,
}

#[derive(Deserialize, Debug)]
pub struct PoolInfo {
    pub id: String,
    pub base_mint: String,
    pub quote_mint: String,
    pub base_vault: String,
    pub quote_vault: String,
    pub market_id: String,
}



pub async fn get_price(token_mint: &str, vs_token: &str, telegram: &TelegramBot) -> Result<f64, Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!(
        "{}?inputMint={}&outputMint={}&amount=1000000",
        env::var("JUPITER_API")?,
        token_mint,
        vs_token
    );
    let response = client.get(&url).send().await?.json::<JupiterQuote>().await?;
    let price = response.data[0].out_amount / 1_000_000.0;
    telegram
        .send_message(&format!("Price for {}: {} SOL", token_mint, price))
        .await?;
    Ok(price)
}

pub async fn monitor_new_pools(telegram: &TelegramBot, tx: tokio::sync::mpsc::Sender<String>) -> Result<(), Box<dyn std::error::Error>> {
    let ws_url = env::var("RPC_WEBSOCKET")?;
    let (mut ws_stream, _) = connect_async(&ws_url).await?;
    // Subscribe to Raydium pool creation events (simplified; requires specific subscription)
    ws_stream.send(Message::Text(r#"{"method":"subscribe","params":{"accounts":["RaydiumProgramId"]}}"#.to_string())).await?;

    while let Some(message) = ws_stream.next().await {
        match message? {
            Message::Text(data) => {
                // Parse WebSocket data for new pools (assumes Raydium event format)
                let pool_id = parse_pool_id(&data).unwrap_or_default();
                if !pool_id.is_empty() {
                    telegram.send_message(&format!("Detected new pool: {}", pool_id)).await?;
                    tx.send(pool_id).await?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn parse_pool_id(data: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(data)
        .ok()
        .and_then(|v| v["result"]["value"]["pubkey"].as_str().map(String::from))
}


pub async fn get_pool_keys(
    token_mint: &str,
    vs_token: &str,
    telegram: &TelegramBot,
) -> Result<PoolInfo, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = env::var("RAYDIUM_POOL_API").map_err(|_| "Missing RAYDIUM_POOL_API in .env")?;
    
    let response = client
        .get(&url)
        .header("User-Agent", "SolanaTradingBot/0.1")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch pool data: {}", e))?;

    if !response.status().is_success() {
        let error_msg = format!("Error fetching pool data: HTTP {}", response.status());
        telegram.send_message(&error_msg).await?;
        return Err(error_msg.into());
    }

    let pools: Vec<PoolInfo> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse pool data: {}", e))?;

    let pool = pools
        .into_iter()
        .find(|p| {
            (p.base_mint == token_mint && p.quote_mint == vs_token) ||
            (p.base_mint == vs_token && p.quote_mint == token_mint)
        })
        .ok_or_else(|| {
            format!("No pool found for {}/{}", token_mint, vs_token)
        })?;

    telegram
        .send_message(&format!("Found pool for {}/{}: {}", token_mint, vs_token, pool.id))
        .await?;

    Ok(pool)
}