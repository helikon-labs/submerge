CREATE TABLE IF NOT EXISTS vote
(
    id                   SERIAL PRIMARY KEY          NOT NULL,
    network_id           INTEGER                     NOT NULL,
    referendum_index     INTEGER                     NOT NULL,
    block_hash           VARCHAR(64)                 NOT NULL,
    extrinsic_index      INTEGER                     NOT NULL,
    extrinsic_hash       VARCHAR(64)                 NOT NULL,
    is_batch             BOOLEAN                     NOT NULL DEFAULT FALSE,
    is_multisig          BOOLEAN                     NOT NULL DEFAULT FALSE,
    is_multisig_executed BOOLEAN                     NOT NULL DEFAULT FALSE,
    is_proxy             BOOLEAN                     NOT NULL DEFAULT FALSE,
    is_successful        BOOLEAN                     NOT NULL DEFAULT FALSE,
    signer_account_id    VARCHAR(64)                 NOT NULL,
    voter_account_id     VARCHAR(64)                 NOT NULL,
    vote_type            VARCHAR(16)                 NOT NULL,
    is_aye               BOOLEAN,
    conviction           INTEGER,
    balance              TEXT,
    aye                  TEXT,
    nay                  TEXT,
    abstain              TEXT,
    created_at           TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT remove_vote_fk_network
        FOREIGN KEY (network_id)
            REFERENCES network (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT vote_fk_referendum
        FOREIGN KEY (network_id, referendum_index)
            REFERENCES referendum (network_id, index)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT vote_fk_block
        FOREIGN KEY (network_id, block_hash)
            REFERENCES block (network_id, hash)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS vote_idx_network_voter
    ON vote (network_id, voter_account_id);

CREATE INDEX vote_idx_network_voter_referendum
    ON vote (network_id, voter_account_id, referendum_index);

CREATE INDEX vote_idx_ordering
    ON vote (network_id ASC, referendum_index ASC);