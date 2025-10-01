CREATE TABLE IF NOT EXISTS referendum_status
(
    id         INTEGER PRIMARY KEY         NOT NULL,
    status     VARCHAR(64)                 NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT referendum_status_u_status UNIQUE (status)
);

INSERT INTO referendum_status (id, status)
VALUES (1, 'Ongoing');
INSERT INTO referendum_status (id, status)
VALUES (2, 'Confirmed');
INSERT INTO referendum_status (id, status)
VALUES (3, 'Rejected');
INSERT INTO referendum_status (id, status)
VALUES (4, 'Cancelled');
INSERT INTO referendum_status (id, status)
VALUES (5, 'Timed Out');
INSERT INTO referendum_status (id, status)
VALUES (6, 'Killed');