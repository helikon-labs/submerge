CREATE TABLE IF NOT EXISTS log
(
    block_hash      BYTEA NOT NULL,
    block_number    BIGINT NOT NULL,
    block_status    BLOCK_STATUS NOT NULL,
    index           INTEGER NOT NULL,
    type            TEXT NOT NULL,
    engine          TEXT,
    data            BYTEA,
    created_at      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT log_pk PRIMARY KEY (block_hash, index),
    CONSTRAINT log_fk_block
        FOREIGN KEY (block_hash)
            REFERENCES block (hash)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS log_idx_block_hash ON log (block_hash);
CREATE INDEX IF NOT EXISTS log_idx_block_number ON log (block_number);