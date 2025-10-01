CREATE TABLE IF NOT EXISTS referendum
(
    network_id            INTEGER                     NOT NULL,
    index                 INTEGER                     NOT NULL,
    track_id              INTEGER                     NOT NULL,
    submission_block_hash VARCHAR(64)                 NOT NULL,
    status_id             INTEGER                     NOT NULL,
    is_retracted          BOOLEAN                     NOT NULL DEFAULT FALSE,
    created_at            TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at            TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT referendum_pk PRIMARY KEY (network_id, index),
    CONSTRAINT referendum_u_cohort_network_index UNIQUE (network_id, index),
    CONSTRAINT referendum_fk_network
        FOREIGN KEY (network_id)
            REFERENCES network (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT referendum_fk_track
        FOREIGN KEY (network_id, track_id)
            REFERENCES track (network_id, id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT referendum_fk_referendum_status
        FOREIGN KEY (status_id)
            REFERENCES referendum_status (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT referendum_fk_submission_block
        FOREIGN KEY (network_id, submission_block_hash)
            REFERENCES block (network_id, hash)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);