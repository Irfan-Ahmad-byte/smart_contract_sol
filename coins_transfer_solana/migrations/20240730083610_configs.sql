-- Sequence for generating unique identifiers for the config table
CREATE SEQUENCE IF NOT EXISTS config_id_seq;

-- Table for storing configuration settings for the application
CREATE TABLE IF NOT EXISTS configs (
    id SMALLINT PRIMARY KEY DEFAULT nextval('config_id_seq'), -- Unique identifier for each configuration (small integer)
    name VARCHAR(128) NOT NULL UNIQUE, -- Name of the configuration setting
    value VARCHAR(128) NOT NULL, -- Value of the configuration setting
    created_at TIMESTAMP NOT NULL DEFAULT NOW(), -- Timestamp when the configuration was created
    updated_at TIMESTAMP NOT NULL DEFAULT NOW() -- Timestamp when the configuration was last updated
    );