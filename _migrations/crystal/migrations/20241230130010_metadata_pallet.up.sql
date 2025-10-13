CREATE TABLE IF NOT EXISTS metadata_pallet
(
    id              SERIAL PRIMARY KEY,
    spec_version    INTEGER NOT NULL,
    index           INTEGER NOT NULL,
    name            VARCHAR(128) NOT NULL,
    created_at      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT metadata_pallet_u_spec_version_index UNIQUE (spec_version, index),
    CONSTRAINT metadata_pallet_fk_metadata
        FOREIGN KEY (spec_version)
            REFERENCES metadata (spec_version)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS metadata_pallet_idx_spec_version ON metadata_pallet (spec_version);
CREATE INDEX IF NOT EXISTS metadata_pallet_idx_gin_name ON metadata_pallet USING GIN (name gin_trgm_ops);