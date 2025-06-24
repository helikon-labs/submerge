CREATE TABLE IF NOT EXISTS event
(
    block_hash      BYTEA NOT NULL,
    block_number    BIGINT NOT NULL,
    block_timestamp BIGINT NOT NULL,
    is_finalized    BOOLEAN NOT NULL,
    module_index    INTEGER NOT NULL,
    module_name     TEXT NOT NULL,
    extrinsic_hash  BYTEA NOT NULL,
    extrinsic_index INTEGER NOT NULL,
    index           INTEGER NOT NULL,
    created_at      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT event_pk PRIMARY KEY (block_hash, index)
);