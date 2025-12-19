DO $$ BEGIN
    IF to_regtype('TRACE_STORAGE_METHOD') IS NULL THEN
        CREATE TYPE TRACE_STORAGE_METHOD
        AS ENUM ('Put', 'ChildPut', 'ChildKill', 'ClearPrefix', 'ChildClearPrefix', 'Append', 'Genesis');
    END IF;
END $$;