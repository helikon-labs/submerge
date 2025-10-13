CREATE TABLE IF NOT EXISTS metadata_error
(
    id              SERIAL PRIMARY KEY,
    pallet_id       INTEGER NOT NULL,
    index           INTEGER NOT NULL,
    name            VARCHAR(128) NOT NULL,
    docs            TEXT[] NOT NULL,
    created_at      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT metadata_error_u_pallet_id_index UNIQUE (pallet_id, index),
    CONSTRAINT metadata_error_fk_metadata_pallet
        FOREIGN KEY (pallet_id)
            REFERENCES metadata_pallet (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS metadata_error_idx_pallet_id ON metadata_error (pallet_id);
CREATE INDEX IF NOT EXISTS metadata_error_idx_name ON metadata_error USING GIN (name gin_trgm_ops);