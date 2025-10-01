CREATE TABLE IF NOT EXISTS track
(
    network_id INTEGER                     NOT NULL,
    id         INTEGER                     NOT NULL,
    name       VARCHAR(64)                 NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT track_pk PRIMARY KEY (network_id, id)
);

INSERT INTO track (network_id, id, name)
VALUES (1, 0, 'Root');

INSERT INTO track (network_id, id, name)
VALUES (1, 1, 'Whitelisted Caller');

INSERT INTO track (network_id, id, name)
VALUES (1, 2, 'Wish For Change');

INSERT INTO track (network_id, id, name)
VALUES (1, 10, 'Staking Admin');

INSERT INTO track (network_id, id, name)
VALUES (1, 11, 'Treasurer');

INSERT INTO track (network_id, id, name)
VALUES (1, 12, 'Lease Admin');

INSERT INTO track (network_id, id, name)
VALUES (1, 13, 'Fellowship Admin');

INSERT INTO track (network_id, id, name)
VALUES (1, 14, 'General Admin');

INSERT INTO track (network_id, id, name)
VALUES (1, 15, 'Auction Admin');

INSERT INTO track (network_id, id, name)
VALUES (1, 20, 'Referendum Canceller');

INSERT INTO track (network_id, id, name)
VALUES (1, 21, 'Referendum Killer');

INSERT INTO track (network_id, id, name)
VALUES (1, 30, 'Small Tipper');

INSERT INTO track (network_id, id, name)
VALUES (1, 31, 'Big Tipper');

INSERT INTO track (network_id, id, name)
VALUES (1, 32, 'Small Spender');

INSERT INTO track (network_id, id, name)
VALUES (1, 33, 'Medium Spender');

INSERT INTO track (network_id, id, name)
VALUES (1, 34, 'Big Spender');


INSERT INTO track (network_id, id, name)
VALUES (2, 0, 'Root');

INSERT INTO track (network_id, id, name)
VALUES (2, 1, 'Whitelisted Caller');

INSERT INTO track (network_id, id, name)
VALUES (2, 2, 'Wish For Change');

INSERT INTO track (network_id, id, name)
VALUES (2, 10, 'Staking Admin');

INSERT INTO track (network_id, id, name)
VALUES (2, 11, 'Treasurer');

INSERT INTO track (network_id, id, name)
VALUES (2, 12, 'Lease Admin');

INSERT INTO track (network_id, id, name)
VALUES (2, 13, 'Fellowship Admin');

INSERT INTO track (network_id, id, name)
VALUES (2, 14, 'General Admin');

INSERT INTO track (network_id, id, name)
VALUES (2, 15, 'Auction Admin');

INSERT INTO track (network_id, id, name)
VALUES (2, 20, 'Referendum Canceller');

INSERT INTO track (network_id, id, name)
VALUES (2, 21, 'Referendum Killer');

INSERT INTO track (network_id, id, name)
VALUES (2, 30, 'Small Tipper');

INSERT INTO track (network_id, id, name)
VALUES (2, 31, 'Big Tipper');

INSERT INTO track (network_id, id, name)
VALUES (2, 32, 'Small Spender');

INSERT INTO track (network_id, id, name)
VALUES (2, 33, 'Medium Spender');

INSERT INTO track (network_id, id, name)
VALUES (2, 34, 'Big Spender');