CREATE TABLE IF NOT EXISTS referendum_event_confirm_aborted
(
    id               SERIAL PRIMARY KEY          NOT NULL,
    network_id       INTEGER                     NOT NULL,
    block_hash       VARCHAR(64)                 NOT NULL,
    referendum_index INTEGER                     NOT NULL,
    created_at       TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT referendum_event_confirm_aborted_u_event UNIQUE (network_id, block_hash, referendum_index),
    CONSTRAINT referendum_event_confirm_aborted_fk_network
        FOREIGN KEY (network_id)
            REFERENCES network (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT referendum_event_confirm_aborted_fk_block
        FOREIGN KEY (network_id, block_hash)
            REFERENCES block (network_id, hash)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT referendum_event_confirm_aborted_fk_referendum
        FOREIGN KEY (network_id, referendum_index)
            REFERENCES referendum (network_id, index)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);