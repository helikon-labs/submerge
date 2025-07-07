CREATE TABLE IF NOT EXISTS metadata
(
    spec_version            INTEGER PRIMARY KEY NOT NULL,
    version                 INTEGER NOT NULL,
    metadata_prefixed_bytes BYTEA NOT NULL,
    metadata_prefixed_json  JSONB NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);