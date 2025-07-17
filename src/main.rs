mod utils;
mod strategies;

use strategies::{sniper::Sniper, grid::Grid, trend::Trend};
use utils::{wallet::Wallet, telegram::{TelegramBot, BotCommand}};
use tokio::sync::mpsc;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let telegram = TelegramBot::new();
    let wallet = Wallet::new(telegram.clone());
    let tokens = vec![
        "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string(), // BONK
        "EKpQGSJtjMFqKZ9u4uhkkR3eFfrk7unuZHKtvsH7BVvb".to_string(), // WIF
        "SOL...".to_string(),
    ];

    let (tx, mut rx) = mpsc::channel::<BotCommand>(100);
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

    // Start Telegram bot
    tokio::spawn(async move {
        telegram.start(tx).await;
    });

    // Handle commands
    while let Some(command) = rx.recv().await {
        match command {
            BotCommand::Start(token) => {
                if tokens.contains(&token) && !active_tokens.contains(&token) {
                    active_tokens.push(token.clone());
                    let (sniper, grid, trend) = strategies.get(&token).unwrap();
                    tokio::spawn({
                        let token = token.clone();
                        let sniper = sniper.clone();
                        async move { sniper.start(token).await.unwrap() }
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
                            }
                        }
                        _ => {
                            telegram.send_message("Invalid strategy").await?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}