use crate::utils::{wallet::Wallet, price_feed::{get_price, monitor_new_pools}};
use crate::strategies::stop_loss::StopLoss;
use crate::utils::telegram::TelegramBot;
use solana_sdk::instruction::Instruction;
use tokio::time::{sleep, Duration};

pub struct Sniper {
    wallet: Wallet,
    telegram: TelegramBot,
    profit_target: f64, // e.g., 0.1 for 10%
}

impl Sniper {
    pub fn new(wallet: Wallet, telegram: TelegramBot) -> Self {
        Sniper {
            wallet,
            telegram,
            profit_target: 0.1,
        }
    }

    pub fn set_profit_target(&mut self, target: f64) {
        self.profit_target = target;
        self.telegram
            .send_message(&format!("Set sniper profit target to {}%", target * 100.0))
            .await.unwrap();
    }

    pub async fn start(&self, token_mint: String) -> Result<(), Box<dyn std::error::Error>> {
        let pools = monitor_new_pools(&self.telegram).await?;
        if pools.contains(&token_mint) {
            let price = get_price(&token_mint, "SOL", &self.telegram).await?;
            // Placeholder: Replace with Raydium swap instruction
            let instruction = Instruction {
                program_id: solana_sdk::pubkey::Pubkey::from_str("RAY...").unwrap(),
                accounts: vec![],
                data: vec![],
            };
            self.wallet.send_transaction(instruction).await?;
            self.telegram
                .send_message(&format!("Sniped {} at {}", token_mint, price))
                .await?;

            // Check profit target
            let profit_price = price * (1.0 + self.profit_target);
            if get_price(&token_mint, "SOL", &self.telegram).await? >= profit_price {
                self.sell(&token_mint, price).await?;
                return Ok(());
            }

            // Start stop-loss
            let stop_loss = StopLoss::new(
                token_mint.clone(),
                price,
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
        }
        Ok(())
    }

    async fn sell(&self, token_mint: &str, price: f64) -> Result<(), Box<dyn std::error::Error>> {
        let instruction = Instruction {
            program_id: solana_sdk::pubkey::Pubkey::from_str("RAY...").unwrap(),
            accounts: vec![],
            data: vec![],
        };
        self.wallet.send_transaction(instruction).await?;
        self.telegram
            .send_message(&format!("Sold {} at profit target: {}", token_mint, price))
            .await?;
        Ok(())
    }
}