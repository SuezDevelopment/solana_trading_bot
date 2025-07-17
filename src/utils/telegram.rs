use teloxide::{prelude::*, types::ChatId};
use std::env;
use log::{info, error};

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

    pub async fn start(&self, commands: tokio::sync::mpsc::Sender<BotCommand>) {
        let bot = self.bot.clone();
        let user_id = self.user_id;

        teloxide::repl(bot, move |msg: Message, bot: Bot| async move {
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
                _ => {
                    bot.send_message(msg.chat.id, "Unknown command").await?;
                }
            }
            Ok(())
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
}