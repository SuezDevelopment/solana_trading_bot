use reqwest::Client;
use serde::Deserialize;
use std::env;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use crate::utils::telegram::TelegramBot;

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
    // Implement parsing logic based on Raydium WebSocket event format
    Some("NEW_POOL_ID".to_string())
}
