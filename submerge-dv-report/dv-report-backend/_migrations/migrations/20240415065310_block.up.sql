CREATE TABLE IF NOT EXISTS block
(
    network_id  INTEGER                     NOT NULL,
    hash        VARCHAR(64)                 NOT NULL,
    number      BIGINT                      NOT NULL,
    timestamp   BIGINT                      NOT NULL,
    parent_hash VARCHAR(64)                 NOT NULL,
    created_at  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT block_pk PRIMARY KEY (network_id, hash)
);

CREATE INDEX IF NOT EXISTS block_idx_hash
    ON block (hash);
CREATE INDEX IF NOT EXISTS block_idx_number
    ON block (number);
CREATE INDEX IF NOT EXISTS block_idx_timestamp
    ON block (timestamp);