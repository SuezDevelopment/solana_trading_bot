use reqwest::Client;
use serde::Deserialize;
use std::env;
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

pub async fn monitor_new_pools(telegram: &TelegramBot) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let client = Client::new();
    let response = client.get(env::var("RAYDIUM_POOL_API")?).send().await?.json::<Vec<RaydiumPool>>().await?;
    let pools: Vec<String> = response.into_iter().map(|pool| pool.id).collect();
    telegram
        .send_message(&format!("Detected {} new pools", pools.len()))
        .await?;
    Ok(pools)
}