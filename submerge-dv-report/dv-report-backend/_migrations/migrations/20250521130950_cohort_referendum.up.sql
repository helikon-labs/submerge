CREATE TABLE IF NOT EXISTS cohort_referendum
(
    network_id          INTEGER NOT NULL,
    cohort_number       INTEGER NOT NULL,
    referendum_index    INTEGER NOT NULL,
    created_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT cohort_referendum_pk PRIMARY KEY (network_id, cohort_number, referendum_index),
    CONSTRAINT cohort_referendum_fk_cohort
        FOREIGN KEY (network_id, cohort_number)
            REFERENCES cohort (network_id, number)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT cohort_referendum_fk_referendum
        FOREIGN KEY (network_id, referendum_index)
            REFERENCES referendum (network_id, index)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS cohort_referendum_idx_network_id_cohort_number
    ON cohort_referendum (network_id, cohort_number);

CREATE INDEX IF NOT EXISTS cohort_referendum_idx_network_id_cohort_number_index_asc
    ON cohort_referendum (network_id, cohort_number, referendum_index ASC);