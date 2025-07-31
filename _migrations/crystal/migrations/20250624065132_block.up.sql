CREATE TABLE IF NOT EXISTS block
(
    hash                BYTEA PRIMARY KEY NOT NULL,
    parent_hash         BYTEA NOT NULL,
    state_root          BYTEA NOT NULL,
    extrinsic_root      BYTEA NOT NULL,
    number              BIGINT NOT NULL,
    timestamp           BIGINT NOT NULL,
    spec_version        INTEGER NOT NULL,
    status              BLOCK_STATUS NOT NULL,
    weight              JSONB,
    extrinsic_count     INTEGER NOT NULL,
    event_count         INTEGER NOT NULL,
    author_account_id   BYTEA NOT NULL,
    created_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
) PARTITION BY HASH (hash);

CREATE INDEX IF NOT EXISTS block_idx_parent_hash
    ON block (parent_hash);
CREATE INDEX IF NOT EXISTS block_idx_number
    ON block (number);
CREATE INDEX IF NOT EXISTS block_idx_timestamp
    ON block (timestamp);
CREATE INDEX IF NOT EXISTS block_idx_spec_version
    ON block (spec_version);
CREATE INDEX IF NOT EXISTS block_idx_number_status
    ON block (number, status);

CREATE TABLE block_0 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 0);
CREATE TABLE block_1 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 1);
CREATE TABLE block_2 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 2);
CREATE TABLE block_3 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 3);
CREATE TABLE block_4 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 4);
CREATE TABLE block_5 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 5);
CREATE TABLE block_6 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 6);
CREATE TABLE block_7 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 7);
CREATE TABLE block_8 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 8);
CREATE TABLE block_9 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 9);
CREATE TABLE block_10 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 10);
CREATE TABLE block_11 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 11);
CREATE TABLE block_12 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 12);
CREATE TABLE block_13 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 13);
CREATE TABLE block_14 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 14);
CREATE TABLE block_15 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 15);
CREATE TABLE block_16 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 16);
CREATE TABLE block_17 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 17);
CREATE TABLE block_18 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 18);
CREATE TABLE block_19 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 19);
CREATE TABLE block_20 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 20);
CREATE TABLE block_21 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 21);
CREATE TABLE block_22 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 22);
CREATE TABLE block_23 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 23);
CREATE TABLE block_24 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 24);
CREATE TABLE block_25 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 25);
CREATE TABLE block_26 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 26);
CREATE TABLE block_27 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 27);
CREATE TABLE block_28 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 28);
CREATE TABLE block_29 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 29);
CREATE TABLE block_30 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 30);
CREATE TABLE block_31 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 31);
CREATE TABLE block_32 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 32);
CREATE TABLE block_33 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 33);
CREATE TABLE block_34 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 34);
CREATE TABLE block_35 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 35);
CREATE TABLE block_36 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 36);
CREATE TABLE block_37 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 37);
CREATE TABLE block_38 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 38);
CREATE TABLE block_39 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 39);
CREATE TABLE block_40 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 40);
CREATE TABLE block_41 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 41);
CREATE TABLE block_42 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 42);
CREATE TABLE block_43 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 43);
CREATE TABLE block_44 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 44);
CREATE TABLE block_45 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 45);
CREATE TABLE block_46 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 46);
CREATE TABLE block_47 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 47);
CREATE TABLE block_48 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 48);
CREATE TABLE block_49 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 49);
CREATE TABLE block_50 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 50);
CREATE TABLE block_51 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 51);
CREATE TABLE block_52 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 52);
CREATE TABLE block_53 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 53);
CREATE TABLE block_54 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 54);
CREATE TABLE block_55 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 55);
CREATE TABLE block_56 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 56);
CREATE TABLE block_57 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 57);
CREATE TABLE block_58 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 58);
CREATE TABLE block_59 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 59);
CREATE TABLE block_60 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 60);
CREATE TABLE block_61 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 61);
CREATE TABLE block_62 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 62);
CREATE TABLE block_63 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 63);
CREATE TABLE block_64 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 64);
CREATE TABLE block_65 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 65);
CREATE TABLE block_66 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 66);
CREATE TABLE block_67 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 67);
CREATE TABLE block_68 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 68);
CREATE TABLE block_69 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 69);
CREATE TABLE block_70 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 70);
CREATE TABLE block_71 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 71);
CREATE TABLE block_72 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 72);
CREATE TABLE block_73 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 73);
CREATE TABLE block_74 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 74);
CREATE TABLE block_75 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 75);
CREATE TABLE block_76 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 76);
CREATE TABLE block_77 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 77);
CREATE TABLE block_78 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 78);
CREATE TABLE block_79 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 79);
CREATE TABLE block_80 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 80);
CREATE TABLE block_81 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 81);
CREATE TABLE block_82 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 82);
CREATE TABLE block_83 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 83);
CREATE TABLE block_84 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 84);
CREATE TABLE block_85 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 85);
CREATE TABLE block_86 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 86);
CREATE TABLE block_87 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 87);
CREATE TABLE block_88 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 88);
CREATE TABLE block_89 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 89);
CREATE TABLE block_90 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 90);
CREATE TABLE block_91 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 91);
CREATE TABLE block_92 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 92);
CREATE TABLE block_93 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 93);
CREATE TABLE block_94 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 94);
CREATE TABLE block_95 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 95);
CREATE TABLE block_96 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 96);
CREATE TABLE block_97 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 97);
CREATE TABLE block_98 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 98);
CREATE TABLE block_99 PARTITION OF block FOR VALUES WITH (modulus 100, remainder 99);