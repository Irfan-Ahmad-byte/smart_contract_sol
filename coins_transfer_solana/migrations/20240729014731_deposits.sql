-- Table for storing user transactions
CREATE TABLE IF NOT EXISTS deposits (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,                  -- Reference to the user
    coin_id SMALLINT NOT NULL,             -- Reference to the coin
    amount DECIMAL(20, 8) NOT NULL,        -- Amount of the coin in the transaction
    fiat_amount DECIMAL(16, 4) NOT NULL,   -- Fiat amount in the transaction
    user_address_id INTEGER NOT NULL,         -- Address associated with the transaction
    transaction_hash VARCHAR(510) NOT NULL,  -- Transaction hash
    status BOOLEAN NOT NULL DEFAULT FALSE, -- Status of the transaction (completed/pending)
    event_id INTEGER,                          -- Event ID for the transaction
    event_status SMALLINT DEFAULT 1,       -- Record status for the transaction
    created_at TIMESTAMP NOT NULL DEFAULT NOW(), -- Timestamp when the transaction was created
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(), -- Timestamp when the transaction was last updated
    FOREIGN KEY (user_id) REFERENCES users (user_id), -- Foreign key to users table
    FOREIGN KEY (coin_id) REFERENCES coins (id),      -- Foreign key to coins table
    FOREIGN KEY (user_address_id) REFERENCES users_wallets (id),      -- Foreign key to users_wallets table
    CONSTRAINT deposits_userid_txhash_uniq UNIQUE (transaction_hash, user_id)   -- Composite UNIQUE constraint
    );