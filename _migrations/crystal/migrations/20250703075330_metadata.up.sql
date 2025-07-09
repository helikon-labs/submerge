CREATE TABLE IF NOT EXISTS metadata
(
    spec_version        INTEGER PRIMARY KEY NOT NULL,
    metadata_version    INTEGER NOT NULL,
    metadata_bytes      BYTEA NOT NULL,
    metadata_json       JSONB NOT NULL,
    created_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);