CREATE TABLE IF NOT EXISTS genesis(
    id          SERIAL PRIMARY KEY,
    key         VARCHAR NOT NULL,
    value       VARCHAR NOT NULL,
    created_at  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT genesis_u_key UNIQUE (key)
);