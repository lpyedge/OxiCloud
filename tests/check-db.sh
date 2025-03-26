#!/bin/bash
# Script to check database state

echo "=== PostgreSQL Database Info ==="
docker exec oxicloud_postgres_1 psql -U postgres -d oxicloud -c "SELECT current_database(), current_user, current_schemas(true);"

echo -e "\n=== Check auth schema exists ==="
docker exec oxicloud_postgres_1 psql -U postgres -d oxicloud -c "SELECT schema_name FROM information_schema.schemata WHERE schema_name = 'auth';"

echo -e "\n=== Check enum type exists ==="
docker exec oxicloud_postgres_1 psql -U postgres -d oxicloud -c "SELECT typname, typnamespace::regnamespace FROM pg_type WHERE typname = 'userrole';"

echo -e "\n=== List tables in auth schema ==="
docker exec oxicloud_postgres_1 psql -U postgres -d oxicloud -c "SELECT table_schema, table_name FROM information_schema.tables WHERE table_schema = 'auth';"

echo -e "\n=== Check users table structure ==="
docker exec oxicloud_postgres_1 psql -U postgres -d oxicloud -c "SELECT column_name, data_type, udt_name FROM information_schema.columns WHERE table_schema = 'auth' AND table_name = 'users' ORDER BY ordinal_position;"

echo -e "\n=== Check users in the database ==="
docker exec oxicloud_postgres_1 psql -U postgres -d oxicloud -c "SELECT id, username, email, role FROM auth.users;"

echo -e "\n=== Check sessions in the database ==="
docker exec oxicloud_postgres_1 psql -U postgres -d oxicloud -c "SELECT id, user_id, expires_at FROM auth.sessions;"