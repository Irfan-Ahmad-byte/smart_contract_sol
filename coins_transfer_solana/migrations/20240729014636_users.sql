-- Sequence for generating unique identifiers for users_id_seq table
CREATE SEQUENCE IF NOT EXISTS users_id_seq START WITH 1 INCREMENT BY 1;

-- Users Table: Stores user information
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY DEFAULT nextval('users_id_seq'),
    user_id INTEGER NOT NULL UNIQUE, -- Unique user identifier
    created_at TIMESTAMP NOT NULL DEFAULT NOW(), -- Timestamp when the user was created
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(), -- Timestamp when the user was last updated
    event_id INTEGER, -- Event ID for the transaction
    event_status SMALLINT DEFAULT 1 -- Record status for the transaction
    );