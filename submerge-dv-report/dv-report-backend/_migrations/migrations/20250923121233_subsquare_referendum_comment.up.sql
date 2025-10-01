CREATE TABLE IF NOT EXISTS subsquare_referendum_comment
(
    id                  VARCHAR(128) PRIMARY KEY,
    network_id          INTEGER                     NOT NULL,
    referendum_index    INTEGER                     NOT NULL,
    referendum_post_id  VARCHAR(128)                NOT NULL,
    reply_to_comment_id VARCHAR(128),
    content             TEXT                        NOT NULL,
    content_type        VARCHAR(128)                NOT NULL,
    content_version     VARCHAR(32)                 NOT NULL,
    author_username     TEXT                        NOT NULL,
    author_public_key   VARCHAR(64),
    author_address      VARCHAR(64)                 NOT NULL,
    author_email_md5    VARCHAR(32),
    height              INTEGER                     NOT NULL,
    created_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    updated_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    data_source         VARCHAR(128)                NOT NULL,
    cid                 VARCHAR(128)                NOT NULL,
    proposer            VARCHAR(64)                 NOT NULL,
    CONSTRAINT ss_referendum_comment_fk_referendum
        FOREIGN KEY (network_id, referendum_index)
            REFERENCES referendum (network_id, index)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT ss_referendum_comment_fk_ss_referendum_comment
        FOREIGN KEY (reply_to_comment_id)
            REFERENCES subsquare_referendum_comment (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS ss_referendum_comment_idx_network_referendum
    ON subsquare_referendum_comment (network_id, referendum_index);
CREATE INDEX IF NOT EXISTS ss_referendum_comment_idx_network_referendum_author_address
    ON subsquare_referendum_comment (network_id, referendum_index, author_address);
CREATE INDEX IF NOT EXISTS ss_referendum_comment_idx_network_referendum_proposer
    ON subsquare_referendum_comment (network_id, referendum_index, proposer);
CREATE INDEX IF NOT EXISTS ss_referendum_comment_idx_author_address
    ON subsquare_referendum_comment (author_address);