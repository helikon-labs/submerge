CREATE TABLE IF NOT EXISTS genesis(
    id          SERIAL PRIMARY KEY,
    key         TEXT NOT NULL,
    value       TEXT NOT NULL,
    created_at  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT genesis_u_key UNIQUE (key)
);

CREATE INDEX IF NOT EXISTS genesis_idx_key ON genesis (key);