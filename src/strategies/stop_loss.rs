use crate::utils::{wallet::Wallet, price_feed::get_price};
use crate::utils::telegram::TelegramBot;
use solana_sdk::instruction::Instruction;

pub struct StopLoss {
    token_mint: String,
    entry_price: f64,
    fixed_stop_loss: f64,
    trailing_stop_loss: f64,
    wallet: Wallet,
    telegram: TelegramBot,
}

impl StopLoss {
    pub fn new(token_mint: String, entry_price: f64, fixed_stop_loss: f64, trailing_stop_loss: f64, wallet: Wallet, telegram: TelegramBot) -> Self {
        StopLoss {
            token_mint,
            entry_price,
            fixed_stop_loss,
            trailing_stop_loss,
            wallet,
            telegram,
        }
    }

    pub async fn check(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let current_price = get_price(&self.token_mint, "SOL", &self.telegram).await?;
        let fixed_threshold = self.entry_price * (1.0 - self.fixed_stop_loss);
        let mut trailing_threshold = self.entry_price;

        if current_price > trailing_threshold {
            trailing_threshold = current_price * (1.0 - self.trailing_stop_loss);
        }

        if current_price <= fixed_threshold || current_price <= trailing_threshold {
            self.sell().await?;
            self.telegram
                .send_message(&format!(
                    "Stop-loss triggered for {} at {}. Fixed: {}, Trailing: {}",
                    self.token_mint, current_price, fixed_threshold, trailing_threshold
                ))
                .await?;
            return Ok(true);
        }
        Ok(false)
    }

    async fn sell(&self) -> Result<(), Box<dyn std::error::Error>> {
        let balance = self.wallet.get_balance(&self.token_mint).await?;
        if balance > 0.0 {
            let instruction = Instruction {
                program_id: solana_sdk::pubkey::Pubkey::from_str("RAY...").unwrap(),
                accounts: vec![],
                data: vec![],
            };
            self.wallet.send_transaction(instruction, &self.token_mint, "sell", get_price(&self.token_mint, "SOL", &self.telegram).await?, balance).await?;
        }
        Ok(())
    }
}