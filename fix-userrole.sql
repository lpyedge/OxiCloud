-- First create the schema if it doesn't exist
CREATE SCHEMA IF NOT EXISTS auth;

-- Output diagnostic information
\echo 'Starting migration fix for auth.userrole'
\echo 'Current schemas:'
\dt auth.*
\echo 'Current types:'
SELECT n.nspname AS schema, t.typname AS type
FROM pg_type t
JOIN pg_catalog.pg_namespace n ON n.oid = t.typnamespace
WHERE n.nspname = 'auth';
\echo '==============================='

-- Check if the type already exists and create it if not
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_type t
        JOIN pg_catalog.pg_namespace n ON n.oid = t.typnamespace
        WHERE t.typname = 'userrole' AND n.nspname = 'auth'
    ) THEN
        -- Create the type
        CREATE TYPE auth.userrole AS ENUM ('admin', 'user');
    END IF;
END
$$;

-- Check if the users table exists and create it if not
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT FROM information_schema.tables 
        WHERE table_schema = 'auth' AND table_name = 'users'
    ) THEN
        -- Create the users table with the proper enum type
        CREATE TABLE auth.users (
            id VARCHAR(36) PRIMARY KEY,
            username VARCHAR(32) NOT NULL UNIQUE,
            email VARCHAR(255) NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            role auth.userrole NOT NULL,
            storage_quota_bytes BIGINT NOT NULL,
            storage_used_bytes BIGINT NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL,
            last_login_at TIMESTAMPTZ,
            active BOOLEAN NOT NULL DEFAULT TRUE
        );
    ELSE
        -- Check if the role column is already auth.userrole type
        IF EXISTS (
            SELECT FROM information_schema.columns
            WHERE table_schema = 'auth' AND table_name = 'users' 
            AND column_name = 'role' AND data_type <> 'USER-DEFINED'
        ) THEN
            -- Try to convert the role column to the new enum type
            BEGIN
                ALTER TABLE auth.users ALTER COLUMN role TYPE auth.userrole USING 
                    CASE WHEN role = 'admin' THEN 'admin'::auth.userrole
                         WHEN role = 'user' THEN 'user'::auth.userrole
                         ELSE 'user'::auth.userrole END;
            EXCEPTION WHEN OTHERS THEN
                RAISE NOTICE 'Error converting role column: %', SQLERRM;
            END;
        END IF;
    END IF;
END
$$;