# User Registry Smart Contract

This crate implements a modular, no-anchor Solana program in Rust providing:

- **User Registration** via a PDA
- **SOL Transfer** instruction
- **SPL Token Transfer** instruction
- **Transaction Validation** (pre/post balance comparison)

## Table of Contents

- [Installation](#installation)
- [Program ID](#program-id)
- [Build & Deploy](#build--deploy)
- [Instruction Reference](#instruction-reference)
  - [RegisterUser](#registeruser)
  - [TransferSol](#transfersol)
  - [TransferSpl](#transferspl)
  - [ValidateTxn](#validatetxn)
- [Account Layout](#account-layout)
- [Error Codes](#error-codes)
- [License](#license)

## Installation

1. Ensure Rust toolchain is installed (stable channel).
2. Install Solana CLI:
   ```bash
   sh -c "$(curl -sSfL https://release.solana.com/v1.17.14/install)"
   ```
3. Clone repository:
   ```bash
   git clone https://github.com/your-org/user_registry.git
   cd user_registry
   ```

## Program ID

Define your program ID constant in `src/lib.rs`:
```rust
use solana_program::declare_id;
declare_id!("YourProgramID111111111111111111111111111111111");
```

## Build & Deploy

Compile and deploy to Devnet:
```bash
# Build
cargo build-bpf --manifest-path=./Cargo.toml -- -- --features=bpf

# Deploy
solana program deploy \
  --program-id ./target/deploy/user_registry-keypair.json \
  ./target/deploy/user_registry.so
```

## Instruction Reference

### RegisterUser
Registers a user account at PDA derived from `["user", payer_pubkey"]`.

- **Accounts:**
  - `[signer]` payer
  - `[writable]` user_account (PDA)
  - `[]` system_program
- **Data:**
  ```rust
  RegistryInstruction::RegisterUser { bump: u8 }
  ```

### TransferSol
Transfers native SOL between two accounts.

- **Accounts:**
  - `[signer]` from
  - `[writable]` to
  - `[]` system_program
- **Data:**
  ```rust
  RegistryInstruction::TransferSol { amount: u64 }
  ```

### TransferSpl
Transfers SPL tokens using CPI to the Token Program.

- **Accounts:**
  - `[signer]` payer (authority)
  - `[]` spl_token::id()
  - `[writable]` from_ata
  - `[writable]` to_ata
  - `[]` mint
- **Data:**
  ```rust
  RegistryInstruction::TransferSpl { amount: u64 }
  ```

### ValidateTxn
Validates an account’s balance change by comparing the current lamports to a provided pre-balance.

- **Accounts:**
  - `[]` account_to_check
- **Data:**
  ```rust
  RegistryInstruction::ValidateTxn { pre_balance: u64 }
  ```

## Account Layout

- **UserAccount** (PDA)
  | Field            | Type     | Size (bytes) |
  | ---------------- | -------- | ------------ |
  | `is_initialized` | `bool`   | 1            |
  | `owner`          | `Pubkey` | 32           |
  | `created_at`     | `u64`    | 8            |

Total: 41 bytes

## Error Codes

Custom program errors mapping to `RegistryError`:

| Code | Description             |
| ---- | ----------------------- |
| 0    | Invalid Instruction     |
| 1    | User already registered |
| 2    | Arithmetic overflow     |

## License

This project is licensed under the MIT License.
