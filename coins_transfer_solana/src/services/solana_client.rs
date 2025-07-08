use bigdecimal::{BigDecimal, FromPrimitive};
use bip39::Mnemonic;
use bitcoincore_rpc::bitcoin::hex::{Case, DisplayHex};
use crypsol_logger::{log, log_custom};
use deadpool_redis::Pool;
use log::Level;
use serde::Deserialize;
use solana_derivation_path::DerivationPath;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{keypair_from_seed_and_derivation_path, Keypair, Signature, Signer};
use solana_sdk::transaction::Transaction;
use solana_system_interface::instruction::transfer;
use solana_transaction_status::{UiTransactionEncoding, EncodedTransaction, UiMessage};
use solana_transaction_status::option_serializer::OptionSerializer;
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token::instruction::transfer_checked;
use sqlx::PgPool;
use walletd_hd_key::prelude::*;
use crate::config::constants::Coin;
use crate::responses::error_msgs::Error;
use crate::services::configs::get_a_config;

pub struct SolanaClient {
    pub endpoint: String,
    pub client: RpcClient,
}

#[derive(Debug, Deserialize)]
pub struct SolanaTransactionInfo {
    pub amount: BigDecimal,
    pub coin: Coin,
    pub network_fee: f64,
    pub recipient: String,
    pub sender: String,
    pub signature: String,
}

impl SolanaClient {
    pub async fn new(pool: &PgPool, redis_pool: &Pool) -> Result<Self, Error> {
        let endpoint = get_a_config(pool, redis_pool, "SOLANA_ADDRESS".to_string()).await?;
        eprintln!("[SOLANA_CLIENT] Using endpoint: {}", endpoint);
        let client = RpcClient::new_with_commitment(endpoint.clone(), CommitmentConfig::confirmed());
        Ok(Self {
            endpoint,
            client,
        })
    }

    pub fn get_endpoint(&self) -> String {
        self.endpoint.clone()
    }

    pub async fn get_transaction_info(&self, signature_str: &str) -> Result<SolanaTransactionInfo, Error> {
        let signature = Signature::from_str(signature_str)
            .map_err(|e| {
                log_custom!(Level::Error, "SOLANA_CLIENT", "Failed to parse signature: {}", e);
                Error::TechnicalIssue
            })?;
        eprintln!("Signature prepared....");
        let info = self.client.get_transaction(&signature, UiTransactionEncoding::Json).await.map_err(|e| {
            log_custom!(Level::Error, "SOLANA_CLIENT", "Failed to get transaction info: {}", e);
            Error::TechnicalIssue
        })?;
        let meta = info.transaction.meta
            .as_ref()
            .ok_or_else(|| { log!(Level::Error, "[Validate Deposit] Missing metadata"); Error::TechnicalIssue })?;

        // 3. Extract account keys and recipient address
        let keys = match &info.transaction.transaction {
            EncodedTransaction::Json(tx) => match &tx.message {
                UiMessage::Raw(raw) => raw.account_keys.clone(),
                _ => return Err(Error::TechnicalIssue),
            },
            _ => return Err(Error::TechnicalIssue),
        };
        let mut recipient_addr = &keys[1];
        let mut sender = &keys[0];
        log!(Level::Info, "Recipient address: {}", recipient_addr);

        // 4. Determine deposit type: SOL native or SPL token
        let (coin, amount_bd) = if meta.post_token_balances.is_none() {
            // Native SOL deposit
            let lamports = meta.post_balances[1].saturating_sub(meta.pre_balances[1]);
            let sol_amount = lamports as f64 / 1_000_000_000.0;
            log!(Level::Info, "Detected SOL deposit: {} lamports ({} SOL)", lamports, sol_amount);
            let bd = BigDecimal::from_f64(sol_amount)
                .ok_or_else(|| { log!(Level::Error, "[Validate Deposit] SOL conversion failed"); Error::TechnicalIssue })?
                .with_scale(9);
            (Coin::Solana, bd)
        } else {
            let pre_token_balances = match meta.pre_token_balances
                .as_ref() {
                OptionSerializer::Some(balances) => balances,
                _ => {
                    log!(Level::Error, "[Validate Deposit] Missing pre_token_balances");
                    return Err(Error::TechnicalIssue);
                }
            };
            let sender_address_entity = match pre_token_balances
                .iter()
                .find(|bal| Coin::from_mint(&bal.mint).is_some()) {
                Some(bal) => bal,
                None => {
                    log!(Level::Error, "[Validate Deposit] No supported token found in pre_token_balances");
                    return Err(Error::TechnicalIssue);
                }
            };

            sender = match sender_address_entity.owner
                .as_ref() {
                OptionSerializer::Some(addr) => addr,
                _ => {
                    log!(Level::Error, "[Validate Deposit] Missing sender address in pre_token_balances");
                    return Err(Error::TechnicalIssue);
                }
            };

            // SPL token deposit
            let balances = meta.post_token_balances
                .as_ref().unwrap();
            let recipient_entry = balances.iter()
                .find(|bal| bal.owner != OptionSerializer::Some(sender.to_string()) && Coin::from_mint(&bal.mint).is_some())
                .ok_or_else(|| { log!(Level::Error, "[Validate Deposit] No supported token found"); Error::TechnicalIssue })?;
            recipient_addr = match recipient_entry.owner.as_ref() {
                OptionSerializer::Some(addr) => addr,
                OptionSerializer::None => {
                    log!(Level::Error, "[Validate Deposit] Missing recipient address in token balance");
                    return Err(Error::TechnicalIssue);
                }
                _ => {
                    log!(Level::Error, "[Validate Deposit] Invalid recipient address format");
                    return Err(Error::TechnicalIssue);
                }
            };
            let coin = Coin::from_mint(&recipient_entry.mint).unwrap();
            let amt = recipient_entry.ui_token_amount.ui_amount
                .ok_or_else(|| { log!(Level::Error, "[Validate Deposit] Missing token amount"); Error::TechnicalIssue })?;
            log!(Level::Info, "Detected {} deposit: {} {}", coin.to_string(), amt, coin.to_string().to_uppercase());
            let bd = BigDecimal::from_f64(amt)
                .ok_or_else(|| { log!(Level::Error, "[Validate Deposit] Token conversion failed"); Error::TechnicalIssue })?
                .with_scale(8);
            (coin, bd)
        };

        Ok(SolanaTransactionInfo {
            amount: amount_bd,
            coin,
            network_fee: meta.fee as f64 / 1_000_000_000.0,
            recipient: recipient_addr.to_string(),
            sender: sender.to_string(),
            signature: signature.to_string(),
        })
    }

    pub async fn transfer_sol(
        &self,
        from: &Keypair,
        to: &Pubkey,
        sols: f64,
    ) -> Result<String, Error> {
        let lamports = (sols * 1_000_000_000.0) as u64; // Convert SOL to lamports
        let recent_blockhash = self.client.get_latest_blockhash().await.unwrap();

        let tx = Transaction::new_signed_with_payer(
            &[transfer(&from.pubkey(), to, lamports)],
            Some(&from.pubkey()),
            &[from],
            recent_blockhash,
        );

        match self.client.send_and_confirm_transaction_with_spinner(&tx).await {
            Ok(signature) => {
                log_custom!(Level::Info, "SOLANA_CLIENT", "Transfer successful: {}", signature);
                Ok(signature.to_string())
            }
            Err(e) => {
                log_custom!(Level::Error, "SOLANA_CLIENT", "Failed to transfer SOL: {}", e);
                Err(Error::TechnicalIssue)
            }
        }
    }

    pub async fn transfer_token(
        &self,
        from: &Keypair,
        to: &Pubkey,
        coin: Coin,
        coin_amount: f64,    // in raw units, e.g. for 1 USDT you'd pass 1_000_000
    ) -> Result<String, Error> {
        let mint = coin.mint().to_string();
        let mint: Pubkey = Pubkey::from_str(&mint).map_err(
            |e| {
                log_custom!(Level::Error, "SOLANA_CLIENT", "Failed to parse mint address: {}", e);
                Error::TechnicalIssue
            }
        )?;

        let decimals: u8 = coin.decimals();

        let amount: u64 = (coin_amount * 10u64.pow(decimals as u32) as f64) as u64;

        let recent_blockhash = match self.client.get_latest_blockhash().await {
            Ok(hash) => hash,
            Err(e) => {
                log_custom!(Level::Error, "SOLANA_CLIENT", "Failed to get latest blockhash: {}", e);
                return Err(Error::TechnicalIssue);
            }
        };
        let payer = &from.pubkey();
        log_custom!(Level::Info, "SOLANA_CLIENT", "Payer pubkey: {}", payer);

        // Derive associated token accounts
        let from_ata = get_associated_token_address(payer, &mint);
        let to_ata   = get_associated_token_address(to, &mint);

        // If recipient ATA doesn't exist, create it
        let mut instructions = vec![];
        // 2. Agar sender ATA nahin exists, create it
        if self.client.get_account(&from_ata).await.is_err() {
            instructions.push(
                create_associated_token_account(
                    &from.pubkey(),       // payer
                    &from.pubkey(),       // owner of the new ATA
                    &mint,                // mint
                    &spl_token::id()
                )
            );
        }

        // 3. Agar recipient ATA nahin exists, create it
        if self.client.get_account(&to_ata).await.is_err() {
            instructions.push(
                create_associated_token_account(
                    &from.pubkey(),       // payer (aap hi SOL fee doge)
                    to,                   // owner = recipient pubkey
                    &mint,
                    &spl_token::id()
                )
            );
        }

        log_custom!(Level::Info, "SOLANA_CLIENT", "Transferring {}  decimals {}", amount, decimals);
        // Transfer checked ensures correct mint and decimals
        let checked_transfer = match transfer_checked(
            &spl_token::id(),
            &from_ata,
            &mint,
            &to_ata,
            &payer,
            &[],
            amount,
            decimals,
        )
        {
            Ok(ix) => ix,
            Err(e) => {
                log_custom!(Level::Error, "SOLANA_CLIENT", "Failed to create transfer instruction: {}", e);
                return Err(Error::TechnicalIssue);
            }
        };
        instructions.push(checked_transfer);

        let tx = Transaction::new_signed_with_payer(
            &instructions,
            Some(payer),
            &[from],
            recent_blockhash,
        );

        match self.client.send_and_confirm_transaction_with_spinner(&tx).await {
            Ok(signature) => {
                log_custom!(Level::Info, "SOLANA_CLIENT", "Token transfer successful: {}", signature);
                Ok(signature.to_string())
            }
            Err(e) => {
                log_custom!(Level::Error, "SOLANA_CLIENT", "Failed to transfer token: {}", e);
                Err(Error::TechnicalIssue)
            }
        }
    }
}

pub async fn get_solana_wallet_keypair(pool: &PgPool, redis_pool: &Pool, account_index: u32) -> Result<(String, String, Keypair), Error> {
    // let mnemonic = Mnemonic::generate_in(Language::English, 24).unwrap();
    // let phrase = mnemonic.to_string();
    let phrase = get_a_config(pool, redis_pool, "MNEMONIC_PHRASE".to_string()).await?;
    let phrase = phrase.trim();
    let mnemonic = Mnemonic::from_str(phrase)
        .map_err(|e| {
            log_custom!(Level::Error, "HD_WALLET", "Failed to create mnemonic: {}", e);
            Error::TechnicalIssue
        })?;

    let seed = mnemonic.to_seed("");
    let seed = Seed::from(seed.as_slice());

    let path = DerivationPath::new_bip44(Some(account_index), Some(0));
    let keypair = keypair_from_seed_and_derivation_path(seed.as_bytes(), Some(path)).map_err(
        |e| {
            log_custom!(Level::Error, "HD_WALLET", "Failed to derive keypair: {}", e);
            Error::TechnicalIssue
        })?;
    
    // store to users wallet in DB
    Ok((keypair.pubkey().to_string(), keypair.secret_bytes().to_hex_string(Case::Upper), keypair))
}