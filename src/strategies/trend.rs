use crate::utils::{wallet::Wallet, price_feed::get_price};
use crate::strategies::stop_loss::StopLoss;
use crate::utils::telegram::TelegramBot;
use solana_sdk::instruction::Instruction;
use ta::indicators::RelativeStrengthIndex;
use tokio::time::{sleep, Duration};

pub struct Trend {
    wallet: Wallet,
    telegram: TelegramBot,
    token_mint: String,
    period: usize,
    rsi_threshold: f64,
}

impl Trend {
    pub fn new(wallet: Wallet, telegram: TelegramBot, token_mint: String, period: usize) -> Self {
        Trend {
            wallet,
            telegram,
            token_mint,
            period,
            rsi_threshold: 30.0,
        }
    }

    pub fn set_rsi_threshold(&mut self, threshold: f64) {
        self.rsi_threshold = threshold;
        self.telegram
            .send_message(&format!("Set RSI threshold for {} to {}", self.token_mint, threshold))
            .await.unwrap();
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut prices = vec![];
        for _ in 0..self.period {
            prices.push(get_price(&self.token_mint, "SOL", &self.telegram).await?);
            sleep(Duration::from_secs(1)).await;
        }

        let mut rsi = RelativeStrengthIndex::new(self.period).unwrap();
        for &price in &prices {
            rsi.next(price);
        }
        let latest_rsi = rsi.next(prices.last().unwrap().clone());

        if latest_rsi < self.rsi_threshold {
            let current_price = get_price(&self.token_mint, "SOL", &self.telegram).await?;
            let instruction = Instruction {
                program_id: solana_sdk::pubkey::Pubkey::from_str("RAY...").unwrap(),
                accounts: vec![],
                data: vec![],
            };
            self.wallet.send_transaction(instruction).await?;
            self.telegram
                .send_message(&format!(
                    "Bought {} at {} (RSI: {})",
                    self.token_mint, current_price, latest_rsi
                ))
                .await?;

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
        }
        Ok(())
    }
}