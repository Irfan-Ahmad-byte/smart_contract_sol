use std::str::FromStr;
use solana_sdk::instruction::AccountMeta;
use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::solana_program::system_program;
use spl_token::id as token_program_id;

// Import your on-chain program's instruction enum and program ID
use smart_contracts_solana::instruction::RegistryInstruction;

fn main() -> Result<()> {
    let program_id = Pubkey::from_str("YourProgramID111111111111111111111111111111111")?;

    // 1) RPC client & payer
    let rpc = RpcClient::new_with_commitment(
        "http://127.0.0.1:8899", // to Replace with RPC URL
        CommitmentConfig::confirmed(),
    );
    let payer = Keypair::from_base58_string(
        "YOUR_BASE58_PRIVATE_KEY"
    );

    // Register user
    let (user_pda, bump) = Pubkey::find_program_address(
        &[b"user", payer.pubkey().as_ref()],
        &program_id,
    );
    let ix_register = Instruction::new_with_borsh(
        program_id,
        &RegistryInstruction::RegisterUser { bump },
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(user_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    send_tx(&rpc, &payer, vec![ix_register])?;
    println!("User registered at PDA: {}", user_pda);

    // Transfer SOL
    let recipient = Pubkey::from_str("Recipient111111111111111111111111111111111")?;
    let ix_sol = Instruction::new_with_borsh(
        program_id,
        &RegistryInstruction::TransferSol { amount: 1_000_000 },
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(recipient, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    send_tx(&rpc, &payer, vec![ix_sol])?;
    println!("0.001 SOL transferred");

    // Transfer SPL Token
    let mint = Pubkey::from_str("TokenMint111111111111111111111111111111111")?;
    let from_ata = get_associated_token_address(&payer.pubkey(), &mint);
    let to_pubkey = Pubkey::from_str("Recipient111111111111111111111111111111111")?;
    let to_ata = get_associated_token_address(&to_pubkey, &mint);
    let ix_spl = Instruction::new_with_borsh(
        program_id,
        &RegistryInstruction::TransferSpl { amount: 10 },
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(token_program_id(), false),
            AccountMeta::new(from_ata, false),
            AccountMeta::new(to_ata, false),
            AccountMeta::new_readonly(mint, false),
        ],
    );
    send_tx(&rpc, &payer, vec![ix_spl])?;
    println!("10 SPL tokens transferred");

    // Validate Transaction Info
    let acct = Pubkey::from_str("AccountToCheck111111111111111111111111111111111")?;
    let pre = rpc.get_balance(&acct)?;
    let ix_val = Instruction::new_with_borsh(
        program_id,
        &RegistryInstruction::ValidateTxn { pre_balance: pre },
        vec![AccountMeta::new_readonly(acct, false)],
    );
    send_tx(&rpc, &payer, vec![ix_val])?;
    println!("Transaction validation invoked");

    Ok(())
}

fn send_tx(
    rpc: &RpcClient,
    payer: &Keypair,
    instructions: Vec<Instruction>,
) -> Result<()> {
    let recent_hash = rpc.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &[payer],
        recent_hash,
    );
    rpc.send_and_confirm_transaction(&tx)?;
    Ok(())
}
