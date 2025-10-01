CREATE TABLE IF NOT EXISTS network
(
    id                  INTEGER PRIMARY KEY         NOT NULL,
    hash                VARCHAR(64)                 NOT NULL,
    chain               VARCHAR(50)                 NOT NULL,
    display             VARCHAR(50)                 NOT NULL,
    ss58_prefix         INTEGER                     NOT NULL,
    token_ticker        VARCHAR(16)                 NOT NULL,
    token_decimal_count INTEGER                     NOT NULL,
    created_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT network_u_hash UNIQUE (hash),
    CONSTRAINT network_u_chain UNIQUE (chain),
    CONSTRAINT network_u_display UNIQUE (display)
);

INSERT INTO network(id, hash, chain, display, ss58_prefix, token_ticker, token_decimal_count)
VALUES (1, '91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3', 'polkadot', 'Polkadot', 0, 'DOT', 10)
ON CONFLICT(id) DO NOTHING;
INSERT INTO network(id, hash, chain, display, ss58_prefix, token_ticker, token_decimal_count)
VALUES (2, 'b0a8d493285c2df73290dfb7e61f870f17b41801197a149ca93654499ea3dafe', 'kusama', 'Kusama', 2, 'KSM', 12)
ON CONFLICT(id) DO NOTHING;