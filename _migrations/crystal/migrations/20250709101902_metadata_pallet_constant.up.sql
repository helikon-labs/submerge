CREATE TABLE IF NOT EXISTS metadata_pallet_constant
(
    spec_version    INTEGER NOT NULL,
    pallet_index    INTEGER NOT NULL,
    pallet_name     VARCHAR(128) NOT NULL,
    index           INTEGER NOT NULL,
    name            VARCHAR(128) NOT NULL,
    type_id         INTEGER,
    value           BYTEA NOT NULL,
    created_at      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT metadata_pallet_constant_pk PRIMARY KEY(spec_version, pallet_index, index),
    CONSTRAINT metadata_pallet_constant_fk_metadata_pallet
        FOREIGN KEY (spec_version, pallet_index)
            REFERENCES metadata_pallet (spec_version, index)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE INDEX metadata_pallet_constant_idx_spec_version ON metadata_pallet_constant (spec_version);
CREATE INDEX metadata_pallet_constant_idx_name ON metadata_pallet_constant (name);