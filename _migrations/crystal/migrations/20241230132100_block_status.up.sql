DO $$ BEGIN
    IF to_regtype('BLOCK_STATUS') IS NULL THEN
        CREATE TYPE BLOCK_STATUS
        AS ENUM ('proposed', 'pruned', 'finalized');
    END IF;
END $$;