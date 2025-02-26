CREATE TABLE IF NOT EXISTS trace_error
(
    block_hash      BYTEA PRIMARY KEY,
    block_number    BIGINT NOT NULL,
    description     TEXT,
    created_at      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS trace_error_idx_number
    ON trace_error (block_number);