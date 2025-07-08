-- Table for storing withdrawal transactions
CREATE TABLE IF NOT EXISTS withdrawals (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL, -- Reference to the user who made the withdrawal
    coin_id SMALLINT NOT NULL, -- Reference to the coin being withdrawn
    usd_amount DECIMAL(20, 8) NOT NULL, -- USD amount of the withdrawal
    coin_amount DECIMAL(20, 8) NOT NULL, -- Coin amount of the withdrawal
    fee_usd_amount DECIMAL(20, 8) NOT NULL, -- USD amount of the withdrawal
    fee_coin_amount DECIMAL(20, 8) NOT NULL, -- Coin amount of the withdrawal
    transaction_hash VARCHAR(510) UNIQUE, -- Unique transaction hash
    address VARCHAR(255), -- Withdrawal address
    status BOOLEAN NOT NULL DEFAULT FALSE, -- Status of the withdrawal (completed/pending)
    event_id INTEGER, -- Event ID for the transaction
    event_status SMALLINT DEFAULT 1, -- Record status for the transaction
    created_at TIMESTAMP NOT NULL DEFAULT NOW(), -- Timestamp when the withdrawal was created
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(), -- Timestamp when the withdrawal was last updated
    FOREIGN KEY (user_id) REFERENCES users (user_id), -- Foreign key reference to users table
    FOREIGN KEY (coin_id) REFERENCES coins (id) -- Foreign key reference to coins table
    );