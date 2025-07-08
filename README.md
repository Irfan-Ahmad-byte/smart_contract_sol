# 🚀 Solana User Registry & Coins Transfer Suite

This mono-repo contains three Rust crates for a complete Solana-based token-management stack:

1. **user_registry** (on-chain program)  
2. **user_registry_client** (off-chain CLI)  
3. **coins_transfer_solana** (Actix-Web microservice)

---

## 📁 Repo Layout

```
.
├── user_registry/               # No-Anchor Solana program
├── user_registry_client/        # CLI client for Registry program
└── coins_transfer_solana/       # Actix-Web token-transfer microservice
```

---

## 1️⃣ user_registry (Smart Contract)

Implements a modular Solana program **without Anchor**, providing:

- **User Registration** (PDA)  
- **SOL Transfer** instruction  
- **SPL Token Transfer** instruction  
- **Transaction Validation** (pre/post balances)  

### Build & Deploy

```bash
cd user_registry
# 1. Build BPF
cargo build-bpf --manifest-path Cargo.toml
# 2. Deploy to Devnet
solana program deploy \
  --program-id ./target/deploy/user_registry-keypair.json \
  ./target/deploy/user_registry.so
```

### Instruction Summary

| Name           | Data                                  | Accounts                                                                                                   |
| -------------- | ------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| RegisterUser   | `{ bump: u8 }`                        | `[signer] payer`, `[writable] user_pda`, `[] system_program`                                               |
| TransferSol    | `{ amount: u64 }`                     | `[signer] from`, `[writable] to`, `[] system_program`                                                      |
| TransferSpl    | `{ amount: u64 }`                     | `[signer] authority`, `[] token_program`, `[writable] from_ata`, `[writable] to_ata`, `[] mint`             |
| ValidateTxn    | `{ pre_balance: u64 }`                | `[] account_to_check`                                                                                      |

---

## 2️⃣ user_registry_client (CLI)

A Rust binary to invoke the `user_registry` program via RPC.

### Features

- Derive PDAs & serialize Borsh instructions  
- Register users  
- Transfer SOL & SPL tokens  
- Validate on-chain balance changes  

### Quickstart

```bash
cd user_registry_client
# configure path to on-chain crate in Cargo.toml:
# user_registry = { path = "../user_registry" }
cargo build --release

# Set your environment:
export PROGRAM_ID=YourProgramID...
export KEYPAIR=~/.config/solana/id.json

# Register user
./target/release/user_registry_client register-user

# Transfer SOL
./target/release/user_registry_client transfer-sol \
  --recipient <PUBKEY> --amount 0.001

# Transfer SPL
./target/release/user_registry_client transfer-spl \
  --mint <TOKEN_MINT> --recipient <PUBKEY> --amount 10

# Validate transaction
./target/release/user_registry_client validate-txn \
  --account <PUBKEY>
```

---

## 3️⃣ coins_transfer_solana (Microservice)

An Actix-Web service to manage deposits/withdrawals for SOL & SPL tokens:

- **User management** (create, rollback)  
- **Coin registry** (SOL, USDT, USDC, etc.)  
- **Wallet address generation & updates**  
- **Deposit & withdrawal** endpoints with webhook validation  
- **History tracking** & **rollback**  

### Tech Stack

- **Rust** (Actix-Web)  
- **Solana-SDK**  
- **PostgreSQL** / **Redis** (for state/caching)  
- **Serde / JSON**  

### Setup & Run

```bash
cd coins_transfer_solana
cargo build --release
cargo run --release
```

#### Key Endpoints

| Group           | Method | Path                                     | Description                       |
| --------------- | ------ | ---------------------------------------- | --------------------------------- |
| **Health**      | GET    | `/api/v1/health`                         | Service liveness check            |
| **Users**       | POST   | `/api/v1/users`                          | Create user                       |
| **Coins**       | GET    | `/api/v1/coins`                          | List supported coins              |
| **Withdrawals** | POST   | `/api/v1/withdrawals`                    | Create withdrawal request         |
| **Deposits**    | POST   | `/api/v1/deposits/usdt/validate_deposit` | Validate USDT deposit webhook     |
| **Configs**     | GET    | `/api/v1/configs`                        | List configurations               |

---

## 📝 License

Each sub-crate is MIT-licensed. See individual `LICENSE` files for details.

---

## 📬 Contributing & Support

- Feel free to open issues or PRs on this repo  
- Maintainer: Irfan Ahmad (github.com/Irfan-Ahmad-byte)  
- For questions, reach out on LinkedIn: linkedin.com/in/irfanahmad-com  

Happy coding! 🚀
