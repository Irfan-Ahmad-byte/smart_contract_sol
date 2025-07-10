# User Registry Client CLI

This crate provides a command-line client to interact with the `user_registry` Solana program.

## Features

- Derive PDAs and invoke on-chain instructions via RPC
- Register a new user
- Transfer native SOL
- Transfer SPL tokens
- Validate transaction (pre/post balance check)

## Table of Contents

- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
  - [Register User](#register-user)
  - [Transfer SOL](#transfer-sol)
  - [Transfer SPL Token](#transfer-spl-token)
  - [Validate Transaction](#validate-transaction)
- [Error Handling](#error-handling)
- [License](#license)

## Prerequisites

- Rust (>=1.60.0)
- Solana CLI (v1.17.14)
- A funded wallet (keypair) on Devnet or Mainnet Beta

## Installation

Clone and build the client:
```bash
git clone https://github.com/your-org/user_registry_client.git
cd user_registry_client
cargo build --release
```

## Configuration

1. Copy or create a Solana keypair file:
   ```bash
   solana-keygen new --outfile ~/.config/solana/id.json
   ```
2. Update your Solana CLI to point to Devnet or Mainnet:
   ```bash
   solana config set --url https://api.devnet.solana.com
   ```
3. Configure the program ID and keypair path in `Config.toml` or environment variables:
   ```toml
   # Config.toml
   program_id = "YourProgramID111111111111111111111111111111111"
   keypair_path = "~/.config/solana/id.json"
   ```

## Usage

Run the binary with subcommands:

```bash
# Register a new user PDA
./target/release/user_registry_client register-user

# Transfer 0.001 SOL 
./target/release/user_registry_client transfer-sol --recipient <RECIPIENT_PUBKEY> --amount 0.001

# Transfer 10 SPL tokens
./target/release/user_registry_client transfer-spl \
  --mint <MINT_ADDRESS> \
  --recipient <RECIPIENT_PUBKEY> \
  --amount 10

# Validate transaction (pre/post balance)
./target/release/user_registry_client validate-txn --account <ACCOUNT_PUBKEY>
```

### register-user
Derives the PDA and sends a `RegisterUser` instruction. Funds the new account.

### transfer-sol
Invokes `TransferSol` with the specified amount in SOL (converted to lamports).

| Option        | Description                  |
| ------------- | ---------------------------- |
| `--recipient` | Destination account pubkey   |
| `--amount`    | Amount in SOL                |

### transfer-spl
Invokes `TransferSpl` for an SPL token transfer.

| Option         | Description                   |
| -------------- | ----------------------------- |
| `--mint`       | SPL token mint address        |
| `--recipient`  | Recipient's pubkey            |
| `--amount`     | Amount in token units (u64)   |

### validate-txn
Fetches the pre-balance, then submits `ValidateTxn` instruction.

| Option      | Description                    |
| ----------- | ------------------------------ |
| `--account` | Pubkey of the account to check |

## Error Handling

All errors are reported via `anyhow::Error` with context. The client will exit with a non-zero exit code on failure.

## License

MIT License.
