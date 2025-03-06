CREATE TABLE IF NOT EXISTS trace
(
    id                  BIGSERIAL NOT NULL,
    block_hash          BYTEA NOT NULL,
    block_parent_hash   BYTEA NOT NULL,
    block_number        BIGINT NOT NULL,
    runtime_version     INT NOT NULL,
    is_finalized        BOOLEAN NOT NULL,
    trace_index         INT NOT NULL,
    key                 TEXT NOT NULL,
    value               TEXT NOT NULL,
    ext_id              TEXT NOT NULL,
    method              VARCHAR(64) NOT NULL,
    parent_id           TEXT,
    created_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    PRIMARY KEY (block_hash, trace_index)
); -- PARTITION BY HASH(block_hash, trace_index);

CREATE INDEX IF NOT EXISTS trace_idx_id
    ON trace (id);
CREATE INDEX IF NOT EXISTS trace_idx_hash
    ON trace (block_hash);
CREATE INDEX IF NOT EXISTS trace_idx_number
    ON trace (block_number);
CREATE INDEX IF NOT EXISTS trace_idx_key
    ON trace (key);

-- CREATE TABLE trace_0 PARTITION OF trace
--     FOR VALUES WITH (MODULUS 2, REMAINDER 0)
--     TABLESPACE n03;

-- CREATE TABLE trace_1 PARTITION OF trace
--     FOR VALUES WITH (MODULUS 2, REMAINDER 1)
--     TABLESPACE n04;