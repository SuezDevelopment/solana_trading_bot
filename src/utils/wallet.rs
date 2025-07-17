use solana_sdk::{
    pubkey::{Pubkey, self},
    signature::{Keypair, Signer},
    transaction::Transaction,
    commitment_config::CommitmentConfig,
};
use solana_client::rpc_client::RpcClient;
use std::env;
use crate::utils::telegram::TelegramBot;

pub struct Wallet {
    keypair: Keypair,
    client: RpcClient,
    telegram: TelegramBot,
}

impl Wallet {
    pub fn new(telegram: TelegramBot) -> Self {
        let private_key = env::var("WALLET_PRIVATE_KEY").expect("Missing WALLET_PRIVATE_KEY");
        let keypair = Keypair::from_base58_string(&private_key);
        let rpc_url = env::var("RPC_ENDPOINT").expect("Missing RPC_ENDPOINT");
        let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
        Wallet { keypair, client, telegram }
    }

    pub async fn send_transaction(&self, instruction: solana_sdk::instruction::Instruction) -> Result<String, Box<dyn std::error::Error>> {
        let recent_blockhash = self.client.get_latest_blockhash()?;
        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.keypair.pubkey()),
            &[&self.keypair],
            recent_blockhash,
        );
        let signature = self.client.send_and_confirm_transaction(&tx)?;
        self.telegram
            .send_message(&format!("Transaction sent: {}", signature))
            .await?;
        Ok(signature.to_string())
    }

    pub async fn get_balance(&self, token_mint: &str) -> Result<f64, Box<dyn std::error::Error>> {
        let mint = Pubkey::from_str(token_mint)?;
        let accounts = self.client.get_token_accounts_by_owner(&self.keypair.pubkey(), mint)?;
        let balance = if accounts.is_empty() {
            0.0
        } else {
            let balance = self.client.get_token_account_balance(&accounts[0].pubkey)?;
            balance.ui_amount.unwrap_or(0.0)
        };
        self.telegram
            .send_message(&format!("Balance for {}: {} tokens", token_mint, balance))
            .await?;
        Ok(balance)
    }

    pub fn clone(&self) -> Self {
        Wallet {
            keypair: Keypair::from_base58_string(&env::var("WALLET_PRIVATE_KEY").unwrap()),
            client: RpcClient::new_with_commitment(
                env::var("RPC_ENDPOINT").unwrap(),
                CommitmentConfig::confirmed(),
            ),
            telegram: TelegramBot::new(),
        }
    }
}