-- Add migration script here
ALTER TABLE users_wallets
ADD COLUMN private_key VARCHAR(500) UNIQUE,
ADD COLUMN wallet_index INT
; -- Private key for the wallet, if applicable