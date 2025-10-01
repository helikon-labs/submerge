CREATE TABLE IF NOT EXISTS polkassembly_referendum_comment
(
    id                  VARCHAR(128) PRIMARY KEY,
    network_id          INTEGER                     NOT NULL,
    referendum_index    INTEGER                     NOT NULL,
    content             TEXT                        NOT NULL,
    updated_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    created_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    proposer            VARCHAR(64) ,
    username            VARCHAR(128) NOT NULL,
    reply_to_comment_id VARCHAR(128),
    CONSTRAINT p_referendum_comment_fk_referendum
        FOREIGN KEY (network_id, referendum_index)
            REFERENCES referendum (network_id, index)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT p_referendum_comment_fk_p_referendum_comment
        FOREIGN KEY (reply_to_comment_id)
            REFERENCES polkassembly_referendum_comment (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS p_referendum_comment_idx_network_referendum
    ON polkassembly_referendum_comment (network_id, referendum_index);
CREATE INDEX IF NOT EXISTS p_referendum_comment_idx_network_referendum_proposer
    ON polkassembly_referendum_comment (network_id, referendum_index, proposer);
CREATE INDEX IF NOT EXISTS p_referendum_comment_idx_proposer
    ON polkassembly_referendum_comment (proposer);