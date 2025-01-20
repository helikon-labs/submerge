CREATE TABLE IF NOT EXISTS block_trace
(
    hash            BYTEA NOT NULL,
    number          BIGINT NOT NULL,
    parent_hash     BYTEA NOT NULL,
    trace_index     INT NOT NULL,
    key             TEXT NOT NULL,
    value           TEXT NOT NULL,
    value_encoded   TEXT,
    ext_id          TEXT NOT NULL,
    method          VARCHAR(64) NOT NULL,
    parent_id       TEXT,
    created_at      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    PRIMARY KEY(hash, trace_index),
    CONSTRAINT block_trace_pk UNIQUE (hash, trace_index)
);

CREATE INDEX IF NOT EXISTS block_trace_idx_hash
    ON block_trace (hash);
CREATE INDEX IF NOT EXISTS block_trace_idx_number
    ON block_trace (number);
CREATE INDEX IF NOT EXISTS block_trace_idx_key
    ON block_trace (key);