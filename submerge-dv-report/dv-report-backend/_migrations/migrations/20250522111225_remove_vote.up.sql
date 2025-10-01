CREATE TABLE IF NOT EXISTS remove_vote
(
    id                   SERIAL PRIMARY KEY          NOT NULL,
    network_id           INTEGER                     NOT NULL,
    referendum_index     INTEGER                     NOT NULL,
    block_hash           VARCHAR(64)                 NOT NULL,
    extrinsic_index      INTEGER                     NOT NULL,
    extrinsic_hash       VARCHAR(64)                 NOT NULL,
    is_batch             BOOLEAN                     NOT NULL,
    is_multisig          BOOLEAN                     NOT NULL,
    is_multisig_executed BOOLEAN                     NOT NULL,
    is_proxy             BOOLEAN                     NOT NULL,
    is_successful        BOOLEAN                     NOT NULL,
    signer_account_id    VARCHAR(64)                 NOT NULL,
    voter_account_id     VARCHAR(64)                 NOT NULL,
    created_at           TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT remove_vote_fk_network
        FOREIGN KEY (network_id)
            REFERENCES network (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT remove_vote_fk_referendum
        FOREIGN KEY (network_id, referendum_index)
            REFERENCES referendum (network_id, index)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT remove_vote_fk_block
        FOREIGN KEY (network_id, block_hash)
            REFERENCES block (network_id, hash)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS remove_vote_idx_network_voter
    ON remove_vote (network_id, voter_account_id);

CREATE INDEX remove_vote_idx_network_voter_referendum
    ON remove_vote (network_id, voter_account_id, referendum_index);