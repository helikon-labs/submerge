DO $$ BEGIN
    IF to_regtype('block_status') IS NULL THEN
        CREATE TYPE block_status
        AS ENUM ('proposed', 'pruned', 'finalized');
    END IF;
END $$;