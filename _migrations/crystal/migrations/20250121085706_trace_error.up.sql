CREATE TABLE IF NOT EXISTS trace_error
(
    id              SERIAL PRIMARY KEY,
    block_hash      BYTEA NOT NULL,
    block_number    BIGINT NOT NULL,
    description     TEXT,
    created_at      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT trace_error_u_hash UNIQUE (block_hash)
);

CREATE INDEX IF NOT EXISTS trace_error_idx_number
    ON trace_error (block_number);