mod utils;
mod strategies;

use strategies::{sniper::Sniper, grid::Grid, trend::Trend};
use utils::{wallet::Wallet, telegram::{TelegramBot, BotCommand}, trade_log::TradeLog, price_feed};
use tokio::sync::mpsc;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let telegram = TelegramBot::new();
    let trade_log = TradeLog::new()?;
    let wallet = Wallet::new(telegram.clone(), trade_log.clone());
    let tokens = vec![
        "A3eME5Ceth4uKS29V4a3eS7Znx2H99v3Hkw3M49eN7jR".to_string(), // PENG
        "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string(), // BONK
        "EKpQGSJtjMFqKZ9u4uhkkR3eFfrk7unuZHKtvsH7BVvb".to_string(), // WIF
        "So11111111111111111111111111111111111111112".to_string(), // SOL
    ];

    let (tx, mut rx) = mpsc::channel::<BotCommand>(100);
    let (pool_tx, pool_rx) = mpsc::channel::<String>(100);
    let mut strategies: HashMap<String, (Sniper, Grid, Trend)> = HashMap::new();
    let mut active_tokens: Vec<String> = vec![];

    for token in &tokens {
        strategies.insert(
            token.clone(),
            (
                Sniper::new(wallet.clone(), telegram.clone()),
                Grid::new(
                    wallet.clone(),
                    telegram.clone(),
                    token.clone(),
                    vec![0.000018, 0.000019, 0.00002, 0.000021],
                    1000.0,
                ),
                Trend::new(wallet.clone(), telegram.clone(), token.clone(), 14),
            ),
        );
    }

    tokio::spawn(async move {
        price_feed::monitor_new_pools(&telegram, pool_tx).await.unwrap();
    });

    // Handle commands
    while let Some(command) = rx.recv().await {
        match command {
            BotCommand::Start(token) => {
                if tokens.contains(&token) && !active_tokens.contains(&token) {
                    active_tokens.push(token.clone());
                    let (sniper, grid, trend) = strategies.get(&token).unwrap();
                    let pool_rx_sniper = pool_rx.clone();
                    tokio::spawn({
                        let token = token.clone();
                        let sniper = sniper.clone();
                        async move { sniper.start(token, pool_rx_sniper).await.unwrap() }
                    });
                    tokio::spawn({
                        let token = token.clone();
                        let grid = grid.clone();
                        async move { grid.start().await.unwrap() }
                    });
                    tokio::spawn({
                        let token = token.clone();
                        let trend = trend.clone();
                        async move { trend.start().await.unwrap() }
                    });
                    telegram.send_message(&format!("Started strategies for {}", token)).await?;
                }
            }
            BotCommand::Stop(token) => {
                if let Some(pos) = active_tokens.iter().position(|t| t == &token) {
                    active_tokens.remove(pos);
                    telegram.send_message(&format!("Stopped strategies for {}", token)).await?;
                }
            }
            BotCommand::Balance(token) => {
                wallet.get_balance(&token).await?;
            }
            BotCommand::Status => {
                let status = if active_tokens.is_empty() {
                    "No active strategies".to_string()
                } else {
                    format!("Active tokens: {:?}", active_tokens)
                };
                telegram.send_message(&status).await?;
            }
            BotCommand::SetParams(token, strategy, key, value) => {
                if let Some((sniper, grid, trend)) = strategies.get_mut(&token) {
                    match strategy.as_str() {
                        "sniper" => {
                            if key == "profit_target" {
                                if let Ok(target) = value.parse::<f64>() {
                                    sniper.set_profit_target(target);
                                }
                            }
                        }
                        "grid" => {
                            if key == "grid_levels" {
                                let levels: Vec<f64> = value
                                    .split(",")
                                    .map(|v| v.parse::<f64>().unwrap())
                                    .collect();
                                grid.set_grid_levels(levels);
                            }
                        }
                        "trend" => {
                            if key == "rsi_threshold" {
                                if let Ok(threshold) = value.parse::<f64>() {
                                    trend.set_rsi_threshold(threshold);
                                }
                            } else if key == "use_ai" {
                                if let Ok(use_ai) = value.parse::<bool>() {
                                    trend.set_use_ai(use_ai);
                                }
                            }
                        }
                        _ => {
                            telegram.send_message("Invalid strategy").await?;
                        }
                    }
                }
            }
            BotCommand::Profit(token) => {
                let current_price = price_feed::get_price(&token, "SOL", &telegram).await?;
                let (profit, percentage) = trade_log.calculate_profit(&token, current_price)?;
                telegram
                    .send_message(&format!(
                        "Profit for {}: ${:.2} ({:.2}%)",
                        token, profit, percentage
                    ))
                    .await?;
            }
        }
    }

    Ok(())
}