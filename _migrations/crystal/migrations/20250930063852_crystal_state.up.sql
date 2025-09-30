CREATE TABLE IF NOT EXISTS crystal_state
(
    id                                  INTEGER PRIMARY KEY,
    last_indexed_finalized_block_number BIGINT,
    last_indexed_finalized_block_hash   BYTEA,
    created_at                          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at                          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

INSERT INTO crystal_state(id, last_indexed_finalized_block_number, last_indexed_finalized_block_hash)
VALUES (1, NULL, NULL)
ON CONFLICT (id) DO NOTHING;