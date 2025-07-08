-- Table for storing user wallet information
CREATE TABLE IF NOT EXISTS users_wallets (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL, -- Reference to the user
    coin_id SMALLINT NOT NULL, -- Reference to the coin
    address VARCHAR(255) UNIQUE, -- Wallet address
    status BOOLEAN NOT NULL DEFAULT FALSE, -- Status of the wallet (active/inactive)
    created_at TIMESTAMP NOT NULL DEFAULT NOW(), -- Timestamp when the wallet was created
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(), -- Timestamp when the wallet was last updated
    FOREIGN KEY (user_id) REFERENCES users (user_id), -- Foreign key reference to users table
    FOREIGN KEY (coin_id) REFERENCES coins (id) -- Foreign key reference to coins table
    );