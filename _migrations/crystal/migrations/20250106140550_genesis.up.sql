CREATE TABLE IF NOT EXISTS genesis(
    id          SERIAL PRIMARY KEY,
    key         BYTEA NOT NULL,
    key_prefix  BYTEA GENERATED ALWAYS AS (substr(key, 1, 32)) STORED,
    value       BYTEA NOT NULL,
    created_at  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT genesis_u_key UNIQUE (key)
);

CREATE INDEX IF NOT EXISTS genesis_idx_key ON genesis (key);
CREATE INDEX IF NOT EXISTS genesis_idx_key_prefix ON genesis (key_prefix);