CREATE TABLE IF NOT EXISTS delegate_comment_account
(
    delegate_id        VARCHAR(64) NOT NULL,
    network_id         INTEGER     NOT NULL,
    comment_account_id VARCHAR(64) NOT NULL,
    CONSTRAINT delegate_comment_account_pk PRIMARY KEY (delegate_id, network_id, comment_account_id),
    CONSTRAINT delegate_comment_account_fk_delegate
        FOREIGN KEY (delegate_id)
            REFERENCES delegate (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT delegate_comment_account_fk_network
        FOREIGN KEY (network_id)
            REFERENCES network (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS delegate_comment_account_idx_delegate
    ON delegate_comment_account (delegate_id, network_id);

-- POLKADOT
-- jid
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('jid',
        1,
        '68a2c9dc9c98e4c3081a06905b76a300f28692a5a4bbdaebfb7adbdc902db343');
-- permanence
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('pdao',
        1,
        'd90e0c661b952f9760e41dab845ac3ace8f13f867e61fbc84c6d6fb68e4cd9b4');
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('pdao',
        1,
        '9d8c441d4baf86f5760e57e06054bea148538c94ac6b354c4aecb6b3e6d74724');
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('pdao',
        1,
        '8489c57973e00401402a0ff4564da4a517f7e9075b4d2e731615be1c8fc66012');
-- hungary
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('hungary',
        1,
        'c07b9222722b0cf30cd9034487a054a00ff3977ac034a7c05caa1b1ac45c8e73');
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('hungary',
        1,
        'aea3ca653928298cd4d1d64cf916aacf72e4e2ca435453941a73327a8dd0a00b');
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('hungary',
        1,
        '580ecbcececdc45961e340519fa57b4e9c33e1c7798c06ddade7309f89822c3b');
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('hungary',
        1,
        'a6796c4e02fca3ded862fce7bab207c1eadac8af1028fcb9e5bf4a0fa3aaae29');
-- polkaworld
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('polkaworld',
        1,
        '4e1550920067048086a9f30f799a1749508e222ad1a9d999f586e6f3e782c932');
-- the kus

-- trustless
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('trustless',
        1,
        '568191edc1aaf4bea93b17cf53ea49ab78e2d25d83dec8581854be93d3bc9609');
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('trustless',
        1,
        '2c24642cef14e77315bf467c00917c749a19c3e5a6df705548a67aa7ad0ad138');
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('trustless',
        1,
        '814daa6bcf9c232db9bf2173447b8ec3bce20835eb4c0ec536e5e9d46d562c60');
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('trustless',
        1,
        'fecc28dd9692d147bb8af57fadbee14659740b6fe50c1775d6941b8a1629b5d8');
-- cybergov
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('trustless',
        1,
        '6a10e2a9b19655add73794c5cc2da8026d6b6bc4cac715ffb67395f9b6ab6f78');
-- daniel olano

-- flez
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('flez',
        1,
        '1ef1e217c5d3f1e9f0c7a0bd001b94f9a4bec21c10d483262cc27fe323e53640');
-- governoun
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('governoun',
        1,
        'a805df3736fde0a8bbcb3c864f28e7051b7c82eea8e30168f744a6ae296c8c4e');
-- le nexus
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('lenexus',
        1,
        '5a49be75ced6618ea80b6368e842fc5bb9e96dc0e474fa0024f3191f391b4d49');
-- pba
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('pba',
        1,
        'd4e03f032676e95763863ee01dead59cd6ec883d825131badda6cece39128127');
-- poland
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('poland',
        1,
        '588145a81030dac4f35bb5d0d7f9537d69db15e1c847a762952d3bc707825c61');
-- reeeeeeeeee
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('reeeeeeeeee',
        1,
        '140af32128f8aadd175d9f24da9b7c251a3ef862bedc8fb50222c207664fc046');
-- saxemberg
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('saxemberg',
        1,
        '00831e9a6121a6d5002d53a89ce1d209d1e3359420a90620a022b22947d41140');
-- twrabbit
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('thewhiterabbit',
        1,
        '7cd5336d50ec2652a90f89edaa708f0952cc7b4e95f16d3e632f4488a4e42423');

-- KUSAMA
-- jid
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('jid',
        2,
        '68a2c9dc9c98e4c3081a06905b76a300f28692a5a4bbdaebfb7adbdc902db343');
-- permanence
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('pdao',
        2,
        '4db3de07f89f1d99c80af30a49057902b56f99f63e7808ed98790c653db04170');
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('pdao',
        2,
        '5eca7d4c4827869bd75a6920a5310e2109bc00636a3fd82d4844e2a732b8bff1');
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('pdao',
        2,
        '8489c57973e00401402a0ff4564da4a517f7e9075b4d2e731615be1c8fc66012');
-- hungary
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('hungary',
        2,
        'a6796c4e02fca3ded862fce7bab207c1eadac8af1028fcb9e5bf4a0fa3aaae29');
-- polkaworld
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('polkaworld',
        2,
        '2c24642cef14e77315bf467c00917c749a19c3e5a6df705548a67aa7ad0ad138');
-- the kus

-- trustless
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('trustless',
        2,
        '186be8005be7b73ee228b4dcb08c17787c68af7a3a44103fad8be49b6821c88e');
-- cybergov
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('trustless',
        2,
        '814daa6bcf9c232db9bf2173447b8ec3bce20835eb4c0ec536e5e9d46d562c60');
-- daniel olano
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('danielolano',
        2,
        '68170716ab7c6735dd0a1012045d9ea33891b5f6596cf97eb217d0962d86a518');
-- flez :: N/A
-- governoun :: N/A
-- le nexus
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('lenexus',
        2,
        '5a49be75ced6618ea80b6368e842fc5bb9e96dc0e474fa0024f3191f391b4d49');
-- pba
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('pba',
        2,
        'd4e03f032676e95763863ee01dead59cd6ec883d825131badda6cece39128127');
-- poland
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('poland',
        2,
        '588145a81030dac4f35bb5d0d7f9537d69db15e1c847a762952d3bc707825c61');
-- reeeeeeeeee
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('reeeeeeeeee',
        2,
        '140af32128f8aadd175d9f24da9b7c251a3ef862bedc8fb50222c207664fc046');
-- saxemberg
INSERT INTO delegate_comment_account (delegate_id, network_id, comment_account_id)
VALUES ('saxemberg',
        2,
        'dea318de2228da64b91c671dd7f325df63959f9c51c86a36f28ec94126f32070');
-- twrabbit :: N/A