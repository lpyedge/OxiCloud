-- Fix the missing UserRole enum type
DO $$
BEGIN
    -- Check if the type already exists
    IF NOT EXISTS (
        SELECT 1 FROM pg_type t
        JOIN pg_catalog.pg_namespace n ON n.oid = t.typnamespace
        WHERE t.typname = 'userrole' AND n.nspname = 'auth'
    ) THEN
        -- Create the type if it doesn't exist
        CREATE TYPE auth.userrole AS ENUM ('admin', 'user');
    END IF;
END
$$;

-- If the table already exists but has a different role column type, 
-- we need to update it to use the new enum type
DO $$
BEGIN
    -- Check if the users table exists
    IF EXISTS (
        SELECT FROM information_schema.tables 
        WHERE table_schema = 'auth' AND table_name = 'users'
    ) THEN
        -- Try to convert the role column to the new enum type
        -- This will work if the column currently contains 'admin' or 'user' values
        BEGIN
            ALTER TABLE auth.users ALTER COLUMN role TYPE auth.userrole USING 
                CASE WHEN role = 'admin' THEN 'admin'::auth.userrole
                     WHEN role = 'user' THEN 'user'::auth.userrole
                     ELSE 'user'::auth.userrole END;
        EXCEPTION WHEN OTHERS THEN
            RAISE NOTICE 'Error converting role column: %', SQLERRM;
        END;
    END IF;
END
$$;