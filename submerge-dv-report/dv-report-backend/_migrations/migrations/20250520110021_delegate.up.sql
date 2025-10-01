CREATE TABLE IF NOT EXISTS delegate
(
    id          VARCHAR(64) PRIMARY KEY     NOT NULL,
    type_id     INTEGER NOT NULL,
    name        TEXT                        NOT NULL,
    short_name  TEXT                        NOT NULL,
    url         TEXT,
    twitter     TEXT,
    email       TEXT,
    created_at  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT delegate_fk_delegate_type_id
        FOREIGN KEY (type_id)
            REFERENCES delegate_type (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
);

INSERT INTO delegate(id, type_id, name, short_name, url, twitter)
VALUES ('pdao', 1, 'Permanence DAO', 'Permanence', 'https://permanence.io', 'PermanenceDAO');

INSERT INTO delegate(id, type_id, name, short_name, url, twitter)
VALUES ('thekus', 1, 'The Kus DAO', 'The Kus', NULL, 'KusDAO');

INSERT INTO delegate(id, type_id, name, short_name, url, twitter)
VALUES ('polkaworld', 1, 'PolkaWorld', 'PolkaWorld', NULL, 'polkaworld_org');

INSERT INTO delegate(id, type_id, name, short_name, url, twitter)
VALUES ('tcore', 1, 'Trustless Core', 'Trustless', NULL, 'trustlesscore');

INSERT INTO delegate(id, type_id, name, short_name, url, twitter)
VALUES ('jid', 1, 'JAM Implementers DAO', 'JID', NULL, NULL);

INSERT INTO delegate(id, type_id, name, short_name, url, twitter)
VALUES ('hungary', 1, 'Polkadot Hungary DAO', 'Hungary', 'https://polkadothungary.net', 'PolkadotHungary');