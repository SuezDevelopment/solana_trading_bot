use crate::utils::{wallet::Wallet, price_feed::get_price};
use crate::strategies::stop_loss::StopLoss;
use crate::utils::telegram::TelegramBot;
use solana_sdk::instruction::Instruction;
use ta::indicators::RelativeStrengthIndex;
use tokio::time::{sleep, Duration};
use reqwest::Client;


pub struct Trend {
    wallet: Wallet,
    telegram: TelegramBot,
    token_mint: String,
    period: usize,
    rsi_threshold: f64,
    use_ai: bool,
}

impl Trend {
    pub fn new(wallet: Wallet, telegram: TelegramBot, token_mint: String, period: usize) -> Self {
        Trend {
            wallet,
            telegram,
            token_mint,
            period,
            rsi_threshold: 30.0,
            use_ai: true,
        }
    }

    pub fn set_rsi_threshold(&mut self, threshold: f64) {
        self.rsi_threshold = threshold;
        self.telegram
            .send_message(&format!("Set RSI threshold for {} to {}", self.token_mint, threshold))
            .await.unwrap();
    }

    pub fn set_use_ai(&mut self, use_ai: bool) {
        self.use_ai = use_ai;
        self.telegram
            .send_message(&format!("Set AI usage for {} to {}", self.token_mint, use_ai))
            .await.unwrap();
    }


    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut prices = vec![];
        for _ in 0..self.period {
            prices.push(get_price(&self.token_mint, "SOL", &self.telegram).await?);
            sleep(Duration::from_secs(1)).await;
        }

        let should_buy = if self.use_ai {
            match get_ai_signal(&self.token_mint).await {
                Ok(signal) => signal == "buy",
                Err(_) => {
                    self.telegram
                        .send_message(&format!("AI signal unavailable for {}, falling back to RSI", self.token_mint))
                        .await?;
                    let mut rsi = RelativeStrengthIndex::new(self.period).unwrap();
                    for &price in &prices {
                        rsi.next(price);
                    }
                    rsi.next(prices.last().unwrap().clone()) < self.rsi_threshold
                }
            }
        } else {
            let mut rsi = RelativeStrengthIndex::new(self.period).unwrap();
            for &price in &prices {
                rsi.next(price);
            }
            rsi.next(prices.last().unwrap().clone()) < self.rsi_threshold
        };

        if should_buy {
            let current_price = get_price(&self.token_mint, "SOL", &self.telegram).await?;
            let instruction = Instruction {
                program_id: solana_sdk::pubkey::Pubkey::from_str("RAY...").unwrap(),
                accounts: vec![],
                data: vec![],
            };
            self.wallet.send_transaction(instruction, &self.token_mint, "buy", current_price, 1000.0).await?;
            self.telegram
                .send_message(&format!(
                    "Bought {} at {} (AI: {}, RSI: {})",
                    self.token_mint, current_price, self.use_ai, self.rsi_threshold
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

    pub fn clone(&self) -> Self {
        Trend {
            wallet: self.wallet.clone(),
            telegram: TelegramBot::new(),
            token_mint: self.token_mint.clone(),
            period: self.period,
            rsi_threshold: self.rsi_threshold,
            use_ai: self.use_ai,
        }
    }
}


async fn get_ai_signal(token_mint: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!(
        "https://api.gmgn.ai/v1/signals/{}",
        token_mint
    );
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", env::var("GMGN_API_KEY")?))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    let signal = response["signal"]
        .as_str()
        .unwrap_or("neutral")
        .to_string();
    Ok(signal)
}