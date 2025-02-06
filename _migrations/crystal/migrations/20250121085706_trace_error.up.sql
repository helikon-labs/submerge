CREATE TABLE IF NOT EXISTS trace_error
(
    hash        BYTEA PRIMARY KEY,
    number      BIGINT NOT NULL,
    description TEXT,
    created_at  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS trace_error_idx_number
    ON trace_error (number);