CREATE TABLE IF NOT EXISTS metadata
(
    spec_version    INTEGER PRIMARY KEY NOT NULL,
    version         INTEGER NOT NULL,
    metadata        BYTEA NOT NULL,
    created_at      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);