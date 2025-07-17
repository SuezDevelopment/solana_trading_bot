use crate::utils::{wallet::Wallet, price_feed::get_price};
use crate::strategies::stop_loss::StopLoss;
use crate::utils::telegram::TelegramBot;
use solana_sdk::instruction::Instruction;
use tokio::time::{sleep, Duration};

pub struct Grid {
    wallet: Wallet,
    telegram: TelegramBot,
    token_mint: String,
    grid_levels: Vec<f64>,
    amount_per_order: f64,
}

impl Grid {
    pub fn new(wallet: Wallet, telegram: TelegramBot, token_mint: String, grid_levels: Vec<f64>, amount_per_order: f64) -> Self {
        Grid {
            wallet,
            telegram,
            token_mint,
            grid_levels,
            amount_per_order,
        }
    }

    pub fn set_grid_levels(&mut self, levels: Vec<f64>) {
        self.grid_levels = levels;
        self.telegram
            .send_message(&format!("Set grid levels for {}: {:?}", self.token_mint, self.grid_levels))
            .await.unwrap();
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let current_price = get_price(&self.token_mint, "SOL", &self.telegram).await?;
        for &level in &self.grid_levels {
            if current_price <= level {
                let instruction = Instruction {
                    program_id: solana_sdk::pubkey::Pubkey::from_str("RAY...").unwrap(),
                    accounts: vec![],
                    data: vec![],
                };
                self.wallet.send_transaction(instruction, &self.token_mint, "buy", level, self.amount_per_order).await?;
                self.telegram
                    .send_message(&format!("Placed buy order for {} at {}", self.token_mint, level))
                    .await?;
            } else if current_price >= level {
                let instruction = Instruction {
                    program_id: solana_sdk::pubkey::Pubkey::from_str_const("RAY...").unwrap(),
                    accounts: vec![],
                    data: vec![],
                };
                self.wallet.send_transaction(instruction, &self.token_mint, "sell", level, self.amount_per_order).await?;
                self.telegram
                    .send_message(&format!("Placed sell order for {} at {}", self.token_mint, level))
                    .await?;
            }
        }

        let stop_loss = StopLoss::new(
            self.token_mint.clone(),
            current_price,
            0.05,
            0.05,
            self.wallet.clone(),
            self.telegram.clone(),
        );
        tokio::spawn(async move {
            loop {
                if stop_loss.check().await.unwrap() {
                    break;
                }
                sleep(Duration::from_secs(60)).await;
            }
        });
        Ok(())
    }

    pub fn clone(&self) -> Self {
        Grid {
            wallet: self.wallet.clone(),
            telegram: TelegramBot::new(),
            token_mint: self.token_mint.clone(),
            grid_levels: self.grid_levels.clone(),
            amount_per_order: self.amount_per_order,
        }
    }
}