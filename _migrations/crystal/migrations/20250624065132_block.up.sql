CREATE TABLE IF NOT EXISTS block
(
    hash                    BYTEA PRIMARY KEY NOT NULL,
    parent_hash             BYTEA NOT NULL,
    state_root              BYTEA NOT NULL,
    extrinsic_root          BYTEA NOT NULL,
    number                  BIGINT NOT NULL,
    timestamp               BIGINT,
    spec_version            INTEGER NOT NULL,
    status                  BLOCK_STATUS NOT NULL,
    weight                  JSONB,
    extrinsic_count         INTEGER NOT NULL,
    event_count             INTEGER NOT NULL,
    author_multi_address    BYTEA,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS block_new_idx_parent_hash
    ON block (parent_hash);
CREATE INDEX IF NOT EXISTS block_new_idx_number
    ON block (number);
CREATE INDEX IF NOT EXISTS block_new_idx_timestamp
    ON block (timestamp);
CREATE INDEX IF NOT EXISTS block_new_idx_spec_version_number
    ON block (spec_version, number);
CREATE INDEX IF NOT EXISTS block_new_idx_number_status
    ON block (number, status);
CREATE INDEX IF NOT EXISTS block_new_idx_author_multi_address
    ON block (author_multi_address);