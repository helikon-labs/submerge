CREATE TABLE IF NOT EXISTS extrinsic
(
    block_hash      BYTEA NOT NULL,
    block_number    BIGINT NOT NULL,
    block_timestamp BIGINT NOT NULL,
    is_finalized    BOOLEAN NOT NULL,
    module_index    INTEGER NOT NULL,
    module_name     TEXT NOT NULL,
    call_index      INTEGER NOT NULL,
    call_name       TEXT NOT NULL,
    hash            BYTEA NOT NULL,
    index           INTEGER NOT NULL,
    nonce           INTEGER NOT NULL,
    signature       BYTEA,
    is_successful   BOOLEAN NOT NULL,
    created_at      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT extrinsic_pk PRIMARY KEY (block_hash, index)
);