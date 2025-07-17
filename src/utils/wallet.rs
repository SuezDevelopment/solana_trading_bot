use solana_sdk::{
    instruction::{ AccountMeta, Instruction },
    pubkey::{ Pubkey, self },
    system_program,
    sysvar::rent,
    signature::{ Keypair, Signer },
    transaction::Transaction,
    commitment_config::CommitmentConfig,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::ID as TOKEN_PROGRAM_ID;

use solana_client::rpc_client::RpcClient;
use std::env;
use crate::utils::{ telegram::TelegramBot, trade_log::TradeLog };

pub struct Wallet {
    keypair: Keypair,
    client: RpcClient,
    telegram: TelegramBot,
    trade_log: TradeLog,
}

impl Wallet {
    pub fn new(telegram: TelegramBot, trade_log: TradeLog) -> Self {
        let private_key = env::var("WALLET_PRIVATE_KEY").expect("Missing WALLET_PRIVATE_KEY");
        let keypair = Keypair::from_base58_string(&private_key);
        let rpc_url = env::var("RPC_ENDPOINT").expect("Missing RPC_ENDPOINT");
        let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
        Wallet { keypair, client, telegram, trade_log }
    }

    pub async fn send_transaction(
        &self,
        instruction: solana_sdk::instruction::Instruction,
        token_mint: &str,
        action: &str,
        price: f64,
        amount: f64
    ) -> Result<String, Box<dyn std::error::Error>> {
        let recent_blockhash = self.client.get_latest_blockhash()?;
        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.keypair.pubkey()),
            &[&self.keypair],
            recent_blockhash
        );
        let signature = self.client.send_and_confirm_transaction(&tx)?;
        self.telegram.send_message(
            &format!(
                "{} {} {} tokens at {} SOL (Tx: {})",
                action,
                token_mint,
                amount,
                price,
                signature
            )
        ).await?;
        self.trade_log.log_trade(token_mint, action, price, amount)?;
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
        self.telegram.send_message(
            &format!("Balance for {}: {} tokens", token_mint, balance)
        ).await?;
        Ok(balance)
    }


    pub async fn ensure_ata(&self, token_mint: &Pubkey) -> Result<Pubkey, Box<dyn std::error::Error>> {
    let ata = get_associated_token_address(&self.keypair.pubkey(), token_mint);
    let account = self.client.get_account(&ata).await;
    if account.is_err() {
        let instruction = spl_associated_token_account::create_associated_token_account(
            &self.keypair.pubkey(),
            &self.keypair.pubkey(),
            token_mint,
            &TOKEN_PROGRAM_ID,
        );
        let recent_blockhash = self.client.get_latest_blockhash()?;
        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.keypair.pubkey()),
            &[&self.keypair],
            recent_blockhash,
        );
        self.client.send_and_confirm_transaction(&tx)?;
        self.telegram
            .send_message(&format!("Created ATA for mint {}", token_mint))
            .await?;
    }
    Ok(ata)
}

    pub fn create_swap_instruction(
        &self,
        pool_info: &super::price_feed::PoolInfo,
        token_mint: &str,
        vs_token: &str,
        amount_in: u64,
        min_amount_out: u64,
        is_base_to_quote: bool
    ) -> Result<Instruction, Box<dyn std::error::Error>> {
        let pool_id = Pubkey::from_str(&pool_info.id)?;
        let base_mint = Pubkey::from_str(&pool_info.base_mint)?;
        let quote_mint = Pubkey::from_str(&pool_info.quote_mint)?;
        let base_vault = Pubkey::from_str(&pool_info.base_vault)?;
        let quote_vault = Pubkey::from_str(&pool_info.quote_vault)?;
        let market_id = Pubkey::from_str(&pool_info.market_id)?;
        let raydium_program = Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8")?;

        let (input_mint, output_mint, input_vault, output_vault) = if is_base_to_quote {
            (base_mint, quote_mint, base_vault, quote_vault)
        } else {
            (quote_mint, base_mint, quote_vault, base_vault)
        };

        let user_input_account = get_associated_token_address(&self.keypair.pubkey(), &input_mint);
        let user_output_account = get_associated_token_address(
            &self.keypair.pubkey(),
            &output_mint
        );

        // Raydium swap instruction data: [instruction_id, amount_in, min_amount_out]
        let data = vec![
            9, // Swap instruction ID for Raydium AMM
            amount_in.to_le_bytes().to_vec(),
            min_amount_out.to_le_bytes().to_vec()
        ].concat();

        let accounts = vec![
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            AccountMeta::new(pool_id, false),
            AccountMeta::new_readonly(
                Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1")?,
                false
            ), // AMM authority
            AccountMeta::new_readonly(market_id, false),
            AccountMeta::new(user_input_account, false),
            AccountMeta::new(user_output_account, false),
            AccountMeta::new(input_vault, false),
            AccountMeta::new(output_vault, false),
            AccountMeta::new_readonly(
                Pubkey::from_str("srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX")?,
                false
            ), // OpenBook program
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(rent::ID, false),
            // Additional accounts (placeholder, adjust as per Raydium's IDL)
            AccountMeta::new_readonly(Pubkey::from_str("11111111111111111111111111111111")?, false),
            AccountMeta::new_readonly(Pubkey::from_str("11111111111111111111111111111111")?, false),
            AccountMeta::new_readonly(Pubkey::from_str("11111111111111111111111111111111")?, false),
            AccountMeta::new_readonly(Pubkey::from_str("11111111111111111111111111111111")?, false),
            AccountMeta::new_readonly(Pubkey::from_str("11111111111111111111111111111111")?, false),
            AccountMeta::new_readonly(Pubkey::from_str("11111111111111111111111111111111")?, false),
            AccountMeta::new_readonly(Pubkey::from_str("11111111111111111111111111111111")?, false)
        ];

        Ok(Instruction {
            program_id: raydium_program,
            accounts,
            data,
        })
    }

    pub fn clone(&self) -> Self {
        Wallet {
            keypair: Keypair::from_base58_string(&env::var("WALLET_PRIVATE_KEY").unwrap()),
            client: RpcClient::new_with_commitment(
                env::var("RPC_ENDPOINT").unwrap(),
                CommitmentConfig::confirmed()
            ),
            telegram: TelegramBot::new(),
            trade_log: TradeLog::new().unwrap(),
        }
    }
}
