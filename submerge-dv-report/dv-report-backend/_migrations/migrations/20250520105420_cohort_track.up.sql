CREATE TABLE IF NOT EXISTS cohort_track
(
    network_id    INTEGER                     NOT NULL,
    cohort_number INTEGER                     NOT NULL,
    track_id      INTEGER                     NOT NULL,
    created_at    TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT cohort_track_u_network_cohort_track UNIQUE (network_id, cohort_number, track_id),
    CONSTRAINT cohort_track_fk_cohort
        FOREIGN KEY (network_id, cohort_number)
            REFERENCES cohort (network_id, number)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT cohort_track_fk_track
        FOREIGN KEY (network_id, track_id)
            REFERENCES track (network_id, id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

INSERT INTO cohort_track(network_id, cohort_number, track_id)
VALUES (1, 4, 2);

INSERT INTO cohort_track(network_id, cohort_number, track_id)
VALUES (1, 4, 11);

INSERT INTO cohort_track(network_id, cohort_number, track_id)
VALUES (1, 4, 30);

INSERT INTO cohort_track(network_id, cohort_number, track_id)
VALUES (1, 4, 31);

INSERT INTO cohort_track(network_id, cohort_number, track_id)
VALUES (1, 4, 32);

INSERT INTO cohort_track(network_id, cohort_number, track_id)
VALUES (1, 4, 33);

INSERT INTO cohort_track(network_id, cohort_number, track_id)
VALUES (1, 4, 34);


INSERT INTO cohort_track(network_id, cohort_number, track_id)
VALUES (2, 4, 2);

INSERT INTO cohort_track(network_id, cohort_number, track_id)
VALUES (2, 4, 11);

INSERT INTO cohort_track(network_id, cohort_number, track_id)
VALUES (2, 4, 30);

INSERT INTO cohort_track(network_id, cohort_number, track_id)
VALUES (2, 4, 31);

INSERT INTO cohort_track(network_id, cohort_number, track_id)
VALUES (2, 4, 32);

INSERT INTO cohort_track(network_id, cohort_number, track_id)
VALUES (2, 4, 33);

INSERT INTO cohort_track(network_id, cohort_number, track_id)
VALUES (2, 4, 34);