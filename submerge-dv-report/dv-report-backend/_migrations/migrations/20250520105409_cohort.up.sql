CREATE TABLE IF NOT EXISTS cohort
(
    network_id        INTEGER                     NOT NULL,
    number            INTEGER                     NOT NULL,
    announcement_date TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    announcement_url  TEXT,
    delegation_date   TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    start_block_hash  VARCHAR(64)                 NOT NULL,
    created_at        TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT cohort_pk PRIMARY KEY (network_id, number),
    CONSTRAINT cohort_fk_network
        FOREIGN KEY (network_id)
            REFERENCES network (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT cohort_fk_start_block
        FOREIGN KEY (network_id, start_block_hash)
            REFERENCES block (network_id, hash)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

INSERT INTO block (network_id, hash, number, timestamp, parent_hash)
VALUES (1, '00aa330acd8b4a114c553c1de41f0c8633d9356e17a42c983d88e4fd7b4b661d', 25571091, 1744619214000,
        '58958231799cf5f8cb6b43ca4106afc84745bb0260b6160e7520a88f0e0d0204');

INSERT INTO cohort(number, network_id, announcement_date, announcement_url, delegation_date, start_block_hash)
VALUES (4, 1, '2025-03-27',
        'https://medium.com/web3foundation/decentralized-voices-cohort-4-delegates-announced-a5a9c64927fd',
        '2025-04-14',
        '00aa330acd8b4a114c553c1de41f0c8633d9356e17a42c983d88e4fd7b4b661d');

INSERT INTO block (network_id, hash, number, timestamp, parent_hash)
VALUES (2, 'e9edfbe4749e1ab8c26a981c956384a6e04d4c966d8d459cdaa8a3c9267bc369', 27921529, 1744619760000,
        '2e0ac3161206f048b1df4cddbea8bd60a093b0e8c444e7fa7eb9a3ff86cac8e3');

INSERT INTO cohort(number, network_id, announcement_date, announcement_url, delegation_date, start_block_hash)
VALUES (4, 2, '2025-03-27',
        'https://medium.com/web3foundation/decentralized-voices-cohort-4-delegates-announced-a5a9c64927fd',
        '2025-04-14',
        'e9edfbe4749e1ab8c26a981c956384a6e04d4c966d8d459cdaa8a3c9267bc369');