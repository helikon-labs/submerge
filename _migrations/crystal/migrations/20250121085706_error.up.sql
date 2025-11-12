CREATE TABLE IF NOT EXISTS error
(
    id              SERIAL PRIMARY KEY,
    block_hash      BYTEA NOT NULL,
    block_number    BIGINT NOT NULL,
    block_status    BLOCK_STATUS NOT NULL,
    description     TEXT,
    created_at      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT trace_error_u_hash UNIQUE (block_hash)
);

CREATE INDEX IF NOT EXISTS error_idx_block_hash
    ON error USING HASH (block_hash);
CREATE INDEX IF NOT EXISTS error_idx_block_number
    ON error (block_number);
CREATE INDEX IF NOT EXISTS error_idx_block_status
    ON error (block_status);