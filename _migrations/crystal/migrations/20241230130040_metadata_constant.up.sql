CREATE TABLE IF NOT EXISTS metadata_constant
(
    id              SERIAL PRIMARY KEY,
    pallet_id       INTEGER NOT NULL,
    index           INTEGER NOT NULL,
    name            VARCHAR(128) NOT NULL,
    type_id         INTEGER,
    type_name       VARCHAR(128) NOT NULL,
    value           BYTEA NOT NULL, -- SCALE-encoded value
    value_json      JSONB,
    docs            TEXT[] NOT NULL,
    created_at      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT metadata_constant_u_pallet_id_index UNIQUE (pallet_id, index),
    CONSTRAINT metadata_constant_fk_metadata_pallet
        FOREIGN KEY (pallet_id)
            REFERENCES metadata_pallet (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS metadata_constant_idx_pallet_id ON metadata_constant (pallet_id);
CREATE INDEX IF NOT EXISTS metadata_constant_idx_name ON metadata_constant USING GIN (name gin_trgm_ops);