-- Sequence for generating unique identifiers for coins table
CREATE SEQUENCE IF NOT EXISTS  coins_id_seq START WITH 1 INCREMENT BY 1;

-- Table for storing information about different coins
CREATE TABLE IF NOT EXISTS coins (
    id SMALLINT PRIMARY KEY DEFAULT nextval('coins_id_seq'), -- Unique identifier for the coin (small integer)
    coin_name VARCHAR(255) NOT NULL UNIQUE, -- Name of the coin
    symbol VARCHAR(255) NOT NULL UNIQUE, -- Symbol of the coin
    status BOOLEAN NOT NULL DEFAULT FALSE, -- Status of the coin (active/inactive)
    created_at TIMESTAMP NOT NULL DEFAULT NOW(), -- Timestamp when the coin was created
    updated_at TIMESTAMP NOT NULL DEFAULT NOW() -- Timestamp when the coin was last updated
    );