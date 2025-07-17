use teloxide::{prelude::*, types::ChatId};
use std::env;
use log::{info, error};
use crate::utils::trade_log::TradeLog;

pub struct TelegramBot {
    bot: Bot,
    user_id: i64,
}

impl TelegramBot {
    pub fn new() -> Self {
        let bot_token = env::var("TELEGRAM_BOT_TOKEN").expect("Missing TELEGRAM_BOT_TOKEN");
        let user_id = env::var("TELEGRAM_USER_ID")
            .expect("Missing TELEGRAM_USER_ID")
            .parse::<i64>()
            .expect("Invalid TELEGRAM_USER_ID");
        TelegramBot {
            bot: Bot::new(bot_token),
            user_id,
        }
    }

    pub async fn send_message(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.bot
            .send_message(ChatId(self.user_id), message)
            .await?;
        Ok(())
    }

    pub async fn start(&self, commands: tokio::sync::mpsc::Sender<BotCommand>, trade_log: TradeLog) {
        let bot = self.bot.clone();
        let user_id = self.user_id;

        teloxide::repl(bot, move |msg: Message, bot: Bot| {
            let trade_log = trade_log.clone();
            async move {
                if msg.from().map(|u| u.id.0 as i64) != Some(user_id) {
                    bot.send_message(msg.chat.id, "Unauthorized user").await?;
                    return Ok(());
                }

                let text = msg.text().unwrap_or("");
                let parts: Vec<&str> = text.split_whitespace().collect();

                if parts.is_empty() {
                    return Ok(());
                }

                match parts[0] {
                    "/start" => {
                        if parts.len() > 1 {
                            let token = parts[1].to_string();
                            commands.send(BotCommand::Start(token)).await.unwrap();
                            bot.send_message(msg.chat.id, format!("Started trading for {}", token)).await?;
                        }
                    }
                    "/stop" => {
                        if parts.len() > 1 {
                            let token = parts[1].to_string();
                            commands.send(BotCommand::Stop(token)).await.unwrap();
                            bot.send_message(msg.chat.id, format!("Stopped trading for {}", token)).await?;
                        }
                    }
                    "/balance" => {
                        if parts.len() > 1 {
                            let token = parts[1].to_string();
                            commands.send(BotCommand::Balance(token)).await.unwrap();
                        }
                    }
                    "/status" => {
                        commands.send(BotCommand::Status).await.unwrap();
                    }
                    "/set_params" => {
                        if parts.len() == 5 {
                            let token = parts[1].to_string();
                            let strategy = parts[2].to_string();
                            let key = parts[3].to_string();
                            let value = parts[4].to_string();
                            commands.send(BotCommand::SetParams(token, strategy, key, value)).await.unwrap();
                            bot.send_message(msg.chat.id, format!("Set {} for {} on {}", key, strategy, token)).await?;
                        }
                    }
                    "/profit" => {
                        if parts.len() > 1 {
                            let token = parts[1].to_string();
                            commands.send(BotCommand::Profit(token)).await.unwrap();
                        }
                    }
                    "/trades" => {
                        if parts.len() > 1 {
                            let token = parts[1].to_string();
                            let limit = parts.get(2).and_then(|s| s.parse::<i64>().ok()).unwrap_or(10);
                            let trades = trade_log.get_trades(&token, limit)?;
                            let message = trades
                                .into_iter()
                                .map(|(mint, action, price, amount, timestamp)| {
                                    format!("{}: {} {} at {} SOL on {}", mint, action, amount, price, timestamp)
                                })
                                .collect::<Vec<_>>()
                                .join("\n");
                            bot.send_message(msg.chat.id, message).await?;
                        }
                    }
                    _ => {
                        bot.send_message(msg.chat.id, "Unknown command").await?;
                    }
                }
                Ok(())
            }
        })
        .await;
    }
}

#[derive(Debug)]
pub enum BotCommand {
    Start(String),
    Stop(String),
    Balance(String),
    Status,
    SetParams(String, String, String, String),
    Profit(String),
}