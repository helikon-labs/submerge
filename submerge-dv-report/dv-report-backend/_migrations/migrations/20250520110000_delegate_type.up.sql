CREATE TABLE IF NOT EXISTS delegate_type
(
    id          INTEGER PRIMARY KEY NOT NULL,
    name        TEXT NOT NULL,
    code        TEXT NOT NULL,
    created_at  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

INSERT INTO delegate_type (id, name, code)
VALUES (1, 'DV DAO', 'dvdao');

INSERT INTO delegate_type (id, name, code)
VALUES (2, 'DV Light', 'dvlight');