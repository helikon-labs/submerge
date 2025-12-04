CREATE TABLE IF NOT EXISTS genesis(
    id                          SERIAL PRIMARY KEY,
    key_prefix                  BYTEA NOT NULL,
    key_params                  BYTEA,
    value                       BYTEA NOT NULL,
    metadata_storage_item_id    INTEGER,
    is_known_key                BOOLEAN NOT NULL,
    created_at                  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT genesis_u_key UNIQUE (key_prefix, key_params)
);

CREATE INDEX IF NOT EXISTS genesis_idx_key_prefix
    ON genesis (key_prefix);
CREATE INDEX IF NOT EXISTS genesis_idx_key_prefix_key_params
    ON genesis (key_prefix, key_params);