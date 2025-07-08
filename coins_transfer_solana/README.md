# 🔁 Coins Transfer API — Solana-based Token Transfer Microservice

A demonstration microservice built with **Rust**, **Actix-Web**, and **Solana-SDK**, allowing users to deposit, withdraw, and manage tokens like **SOL**, **USDT**, **USDC**, and other supported SPL tokens on the **Solana blockchain**.

---

## 🚀 Features

- ⚙️ **Configurable coin management**
- 🪙 **Token transfers & deposits** (SOL, USDT, USDC, etc.)
- 🧾 **Wallet management** (address generation, updates)
- 📈 **Conversion rate fetcher**
- 🔁 **Rollback mechanism** for user creation and withdrawals
- 🧠 **Webhook validation** for incoming USDT deposits
- 📊 **Deposit & Withdrawal history tracking**
- ❤️ **Built in Rust using high-performance Actix-Web**

---

## 📁 API Endpoints

### ✅ Health
| Method | Endpoint        | Description         |
|--------|------------------|---------------------|
| GET/POST | `/api/v1/health` | Health check        |

### 👤 Users
| Method | Endpoint                        | Description               |
|--------|----------------------------------|---------------------------|
| POST   | `/api/v1/users/`                | Create a new user         |
| POST   | `/api/v1/users/rollback/{id}`   | Rollback user creation    |

### 🪙 Coins
| Method | Endpoint                         | Description                 |
|--------|-----------------------------------|-----------------------------|
| POST   | `/api/v1/coins/`                 | Add a supported coin        |
| GET    | `/api/v1/coins/`                 | List all coins              |
| GET    | `/api/v1/coins/rate/{symbol}`    | Get token conversion rate   |
| GET    | `/api/v1/coins/address/new`      | Generate new token address  |

### 💼 Wallets
| Method | Endpoint                              | Description               |
|--------|----------------------------------------|---------------------------|
| GET    | `/api/v1/wallets/`                    | Get wallet info           |
| PUT    | `/api/v1/wallets/{wallet_id}`         | Update wallet             |

### 💸 Withdrawals
| Method | Endpoint                                     | Description                     |
|--------|-----------------------------------------------|---------------------------------|
| POST   | `/api/v1/withdrawals/`                       | Create a withdrawal request     |
| POST   | `/api/v1/withdrawals/rollback/{event_id}`    | Rollback a withdrawal request   |
| GET    | `/api/v1/withdrawals/history`                | Get withdrawal history          |

### 💰 Deposits
| Method | Endpoint                                     | Description                      |
|--------|-----------------------------------------------|----------------------------------|
| POST   | `/api/v1/deposits/usdt/validate_deposit`     | Validate USDT deposit webhook    |
| GET    | `/api/v1/deposits/history`                   | Get deposit history              |

### ⚙️ Configs
| Method | Endpoint                      | Description                  |
|--------|-------------------------------|------------------------------|
| POST   | `/api/v1/configs/`            | Create config                |
| GET    | `/api/v1/configs/`            | List all configs             |
| GET    | `/api/v1/configs/{name}`      | Get config by name           |
| PUT    | `/api/v1/configs/{name}`      | Update config by name        |
| DELETE | `/api/v1/configs/{name}`      | Delete config by name        |
| DELETE | `/api/v1/configs/`            | Delete all configs           |

---

## 🧰 Tech Stack

- **Rust** — memory-safe and blazingly fast backend
- **Actix-Web** — powerful, asynchronous web framework
- **Solana-SDK** — to interact with Solana blockchain
- **Serde / JSON** — for payloads and serialization
- **PostgreSQL / Redis** *(assumed for production use)*

---

## 🔧 Setup Instructions

> _Ensure you have Rust (>= 1.70), Cargo, and Solana CLI installed._

```bash
# Clone the repo
git clone https://github.com/Irfan-Ahmad-byte/coins_transfer_solana.git
cd coins_transfer_solana

# Build and run
cargo build
cargo run
```

## 📬 Contact & Contributions
Feel free to reach out for feedback, contributions, or questions!

- GitHub: github.com/Irfan-Ahmad-byte

- LinkedIn: linkedin.com/in/irfanahmad-com


## 📜 License
This project is for demonstration purposes. You are free to fork, extend, and use it with appropriate attribution.