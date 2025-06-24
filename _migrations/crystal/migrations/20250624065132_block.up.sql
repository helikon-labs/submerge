CREATE TABLE IF NOT EXISTS block
(
    hash            BYTEA PRIMARY KEY NOT NULL,
    parent_hash     BYTEA NOT NULL,
    state_root      BYTEA NOT NULL,
    extrinsic_root  BYTEA NOT NULL,
    number          BIGINT NOT NULL,
    timestamp       BIGINT NOT NULL,
    runtime_version INTEGER NOT NULL,
    is_finalized    BOOLEAN NOT NULL,
    extrinsic_count INTEGER NOT NULL,
    event_count     INTEGER NOT NULL,
    created_at      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS block_idx_number
    ON block (number);