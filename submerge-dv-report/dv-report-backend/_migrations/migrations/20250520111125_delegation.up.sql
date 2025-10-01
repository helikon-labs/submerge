CREATE TABLE IF NOT EXISTS delegation
(
    id                    SERIAL PRIMARY KEY          NOT NULL,
    cohort_number         INTEGER                     NOT NULL,
    network_id            INTEGER                     NOT NULL,
    delegator_account_id  VARCHAR(64)                 NOT NULL,
    delegate_id           VARCHAR(64)                 NOT NULL,
    delegate_account_id   VARCHAR(64)                 NOT NULL,
    start_block_hash      VARCHAR(64)                 NOT NULL,
    start_extrinsic_hash  VARCHAR(64)                 NOT NULL,
    start_extrinsic_index INTEGER                     NOT NULL,
    end_block_hash        VARCHAR(64),
    end_extrinsic_hash    VARCHAR(64),
    end_extrinsic_index   INTEGER,
    created_at            TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT delegation_u_cohort_network_delegator UNIQUE (cohort_number, network_id, delegator_account_id),
    CONSTRAINT delegation_u_cohort_network_delegate UNIQUE (cohort_number, network_id, delegate_id),
    CONSTRAINT delegation_fk_network
        FOREIGN KEY (network_id)
            REFERENCES network (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT delegation_fk_delegate
        FOREIGN KEY (delegate_id)
            REFERENCES delegate (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT delegation_fk_start_block
        FOREIGN KEY (network_id, start_block_hash)
            REFERENCES block (network_id, hash)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT delegation_fk_end_block
        FOREIGN KEY (network_id, end_block_hash)
            REFERENCES block (network_id, hash)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS delegation_idx_network_delegate
    ON delegation (network_id, delegate_id);

INSERT INTO block (network_id, hash, number, timestamp, parent_hash)
VALUES (1, 'f08e99db347e207c0539d1c7c8dc4ab4b443c3fb74c5d8bef8722f71e5b43edb', 25571026, 1744618824000,
        '0dd1c16f0c999a5501bf34a395eab60849749a04d590f7a15c3c86142af53cc0');

INSERT INTO delegation (cohort_number, network_id, delegator_account_id, delegate_id, delegate_account_id,
                        start_block_hash,
                        start_extrinsic_hash, start_extrinsic_index)
VALUES (4,
        1,
        'd10577dd7d364b294d2e9a0768363ac885efb8b1c469da6c4f2141d4f6560c1f',
        'pdao',
        '9d8c441d4baf86f5760e57e06054bea148538c94ac6b354c4aecb6b3e6d74724',
        'f08e99db347e207c0539d1c7c8dc4ab4b443c3fb74c5d8bef8722f71e5b43edb',
        '969a6895c9e2b8b3564fa09adee57944411984fa80f3ed24add3e1fe02b1f1db',
        2);

INSERT INTO block (network_id, hash, number, timestamp, parent_hash)
VALUES (1, '2417a7735f1858b3a55691dfb3b31374bf814670fd96410e53a9412f887bd818', 25571048, 1744618956001,
        '040c9695cf2e37816804fd678fbca436979d29e62ae15ad5cfae17b8999b5fd3');

INSERT INTO delegation (cohort_number, network_id, delegator_account_id, delegate_id, delegate_account_id,
                        start_block_hash,
                        start_extrinsic_hash, start_extrinsic_index)
VALUES (4,
        1,
        '6c1b752375304917c15af9c2e7a4426b3af513054d89f6c7bb26cd7e30e4413e',
        'polkaworld',
        '4e1550920067048086a9f30f799a1749508e222ad1a9d999f586e6f3e782c932',
        '2417a7735f1858b3a55691dfb3b31374bf814670fd96410e53a9412f887bd818',
        'dc91c2b4b5cf26ccc8325ab186a746b1c10d19bbf52e3dc891d97ee372fda900',
        2);

INSERT INTO block (network_id, hash, number, timestamp, parent_hash)
VALUES (1, '8c528920987e5a80b17f08c82466b63da45154e0cb98f48dc4d795b091f38500', 25571060, 1744619028000,
        '7da12d33619b0422dbf2e19c2bd4cd78cae612a8cde2016e1640598a2f6c2e21');

INSERT INTO delegation (cohort_number, network_id, delegator_account_id, delegate_id, delegate_account_id,
                        start_block_hash,
                        start_extrinsic_hash, start_extrinsic_index)
VALUES (4,
        1,
        '9561809d76c46eaad3f19d2d392e0a4962086ce116a8739fe7d458bdc3bd4f1d',
        'tcore',
        '814daa6bcf9c232db9bf2173447b8ec3bce20835eb4c0ec536e5e9d46d562c60',
        '8c528920987e5a80b17f08c82466b63da45154e0cb98f48dc4d795b091f38500',
        '334e69a981e9f0e54ba7f25072901b6cf7dc88981b36748af95a76834ac94a93',
        2);

INSERT INTO delegation (cohort_number, network_id, delegator_account_id, delegate_id, delegate_account_id,
                        start_block_hash,
                        start_extrinsic_hash, start_extrinsic_index)
VALUES (4,
        1,
        'e8e2262e16583379847ad70b7f77ca559d5c17aa69062230f8a0dbd1bf5da5d4',
        'jid',
        '68a2c9dc9c98e4c3081a06905b76a300f28692a5a4bbdaebfb7adbdc902db343',
        '00aa330acd8b4a114c553c1de41f0c8633d9356e17a42c983d88e4fd7b4b661d',
        'bd5951b7c438571700028329ca3842562392622c14d544ffa47288f8a26f6f8b',
        4);

INSERT INTO block (network_id, hash, number, timestamp, parent_hash)
VALUES (1, 'e59ba818d2c54c02fcc93775b95e46a9619df16c59b2c64908dbc71d8384dd7c', 23364955, 1731328740000,
        'ada98c364f3ff84fda4f6a7cd40a060ebdfc04388e0ee54d98a0372c835d2be1');

INSERT INTO delegation (cohort_number, network_id, delegator_account_id, delegate_id, delegate_account_id,
                        start_block_hash,
                        start_extrinsic_hash, start_extrinsic_index)
VALUES (4,
        1,
        '83bf40ac1231b8b9b539abead87569ae512edd874c710cd249afecab1093cf03',
        'thekus',
        'bee2c0254a1998a65fc2787a82cbecdca2c0a675be63ae9c5148e32ae753d01f',
        'e59ba818d2c54c02fcc93775b95e46a9619df16c59b2c64908dbc71d8384dd7c',
        'e0df958078d8e6baebed80e6bc472f5c27e40860ec5ba4ad944df3fcb45edc27',
        2);

INSERT INTO delegation (cohort_number, network_id, delegator_account_id, delegate_id, delegate_account_id,
                        start_block_hash,
                        start_extrinsic_hash, start_extrinsic_index)
VALUES (4,
        1,
        '429b067ff314c1fed75e57fcf00a6a4ff8611268e75917b5744ac8c4e1810d17',
        'hungary',
        '840d10a738d1185e5e30367ae1f9e663f3604e9d1e99b38013ecf26e9c0e9251',
        'e59ba818d2c54c02fcc93775b95e46a9619df16c59b2c64908dbc71d8384dd7c',
        'e0df958078d8e6baebed80e6bc472f5c27e40860ec5ba4ad944df3fcb45edc27',
        2);



INSERT INTO block (network_id, hash, number, timestamp, parent_hash)
VALUES (2, '6f85590ce2718574ece8b9468944c9888f724ff21fab7c0a4c3e4ff29b9870ef', 27921280, 1744618254000,
        '6762df1ede09aba402d0c4d1245aa4e93705b8ffa2cad7b735632dd894c9fe8e');

INSERT INTO delegation (cohort_number, network_id, delegator_account_id, delegate_id, delegate_account_id,
                        start_block_hash,
                        start_extrinsic_hash, start_extrinsic_index)
VALUES (4,
        2,
        '560e6196f4ed72438184f7be3657c7df91d0374f9e39a5be53b0b86c2d80b979',
        'pdao',
        '4db3de07f89f1d99c80af30a49057902b56f99f63e7808ed98790c653db04170',
        '6f85590ce2718574ece8b9468944c9888f724ff21fab7c0a4c3e4ff29b9870ef',
        '06859eaf47b104039f5a366cb12bd85ab639d50615cb3b8a8965979b840a8a82',
        2);

INSERT INTO block (network_id, hash, number, timestamp, parent_hash)
VALUES (2, '7d4a53db9774813285a5fd2656fdc08daefcce0d8e3f2888facf2fc180605c8d', 27921459, 1744619340000,
        'e876039340df06fc46a42ee87d70a124b2c847fd10f064d21902a5df30742481');

INSERT INTO delegation (cohort_number, network_id, delegator_account_id, delegate_id, delegate_account_id,
                        start_block_hash,
                        start_extrinsic_hash, start_extrinsic_index)
VALUES (4,
        2,
        '982f5263679b39960e3b787a0330f8d35c26a9bd2f2caa2f21712c5e36235ceb',
        'polkaworld',
        '2c24642cef14e77315bf467c00917c749a19c3e5a6df705548a67aa7ad0ad138',
        '7d4a53db9774813285a5fd2656fdc08daefcce0d8e3f2888facf2fc180605c8d',
        '2a466f03b5fc95934ad4a8fa07b3b63beb103167975959552a64bbf78dd0f6a5',
        2);

INSERT INTO block (network_id, hash, number, timestamp, parent_hash)
VALUES (2, 'e3bf8cc4fc8f403485464e39e243f039de91200b534b0cfeae8721dddefebd53', 27921516, 1744619682000,
        '49177671a64798d879305470c1258d9e7b2a35903496998cb283ea584f601fa4');

INSERT INTO delegation (cohort_number, network_id, delegator_account_id, delegate_id, delegate_account_id,
                        start_block_hash,
                        start_extrinsic_hash, start_extrinsic_index)
VALUES (4,
        2,
        '1a33bc5ddf8e989a9413b9c1ed4df64b9f2479d3bb8b2e40a419b7f2dd3470e4',
        'tcore',
        '40e83c5c579054aba747e333f5d0c48924ef3228709c245dd23f606d1b2a81d2',
        'e3bf8cc4fc8f403485464e39e243f039de91200b534b0cfeae8721dddefebd53',
        'f1895cc23cdf67f6e0d7b18a953118a1c77273196694024e619b3d151a21f60c',
        2);

INSERT INTO delegation (cohort_number, network_id, delegator_account_id, delegate_id, delegate_account_id,
                        start_block_hash,
                        start_extrinsic_hash, start_extrinsic_index)
VALUES (4,
        2,
        '90fcc36da703c17c329100a8303dc0c3f30adc1c53885abeafd8d02264131005',
        'jid',
        '68a2c9dc9c98e4c3081a06905b76a300f28692a5a4bbdaebfb7adbdc902db343',
        'e9edfbe4749e1ab8c26a981c956384a6e04d4c966d8d459cdaa8a3c9267bc369',
        '7cf8a207b24e1738ddd72effcc4014c75c5e86465ee6fd9893af54f822e9a217',
        4);

INSERT INTO block (network_id, hash, number, timestamp, parent_hash)
VALUES (2, '9660ed2ee20297170b1e61e59a33e447c65db3632cf0d6d459880548630c71d9', 25732465, 1731321798000,
        '5389aca2e2bb492d1fb1fd4a3d5b167f0e8247fcc0d9e409f76bc224152b84d7');

INSERT INTO delegation (cohort_number, network_id, delegator_account_id, delegate_id, delegate_account_id,
                        start_block_hash,
                        start_extrinsic_hash, start_extrinsic_index)
VALUES (4,
        2,
        'feaaab56ff00794a89916eadd949f9d76ee42de59554b5435ba4b8163a962610',
        'thekus',
        'fc8da8cd554b5256f00c5e43f8a30b62f77e2d9b34730eb823e819e141c029b1',
        '9660ed2ee20297170b1e61e59a33e447c65db3632cf0d6d459880548630c71d9',
        'd9ad4ff269978c894c24af6344c0176616a13f471ccc3b04945ec3e6fb9b6971',
        2);

INSERT INTO delegation (cohort_number, network_id, delegator_account_id, delegate_id, delegate_account_id,
                        start_block_hash,
                        start_extrinsic_hash, start_extrinsic_index)
VALUES (4,
        2,
        'e1dccec9bfccbc19f26e7ba8c78ae87663a713b80d8c63a2c18a1b5823777488',
        'hungary',
        'e216c8e8d08097487ee17d41f701c3c4e7d207a97c8ba691a755fb01a25db835',
        '9660ed2ee20297170b1e61e59a33e447c65db3632cf0d6d459880548630c71d9',
        'd9ad4ff269978c894c24af6344c0176616a13f471ccc3b04945ec3e6fb9b6971',
        2);