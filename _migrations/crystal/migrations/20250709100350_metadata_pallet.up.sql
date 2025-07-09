CREATE TABLE IF NOT EXISTS metadata_pallet
(
    spec_version    INTEGER NOT NULL,
    index           INTEGER NOT NULL,
    name            VARCHAR(128) NOT NULL,
    created_at      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT metadata_pallet_pk PRIMARY KEY(spec_version, index),
    CONSTRAINT metadata_pallet_fk_metadata
        FOREIGN KEY (spec_version)
            REFERENCES metadata (spec_version)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX metadata_pallet_idx_spec_version ON metadata_pallet (spec_version);
CREATE INDEX metadata_pallet_idx_name ON metadata_pallet (name);