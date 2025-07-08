-- Table for storing conversion rates of coins
CREATE TABLE IF NOT EXISTS conversion_rates (
    id SERIAL PRIMARY KEY,
    coin_id SMALLINT NOT NULL, -- Reference to the coin
    conversion_rate DECIMAL(20, 8) NOT NULL, -- Conversion rate of the coin
    created_at TIMESTAMP NOT NULL DEFAULT NOW(), -- Timestamp when the conversion rate was created
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(), -- Timestamp when the conversion rate was last updated
    FOREIGN KEY (coin_id) REFERENCES coins (id) -- Foreign key reference to coins table
    );