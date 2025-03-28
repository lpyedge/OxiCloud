#!/bin/bash

# This script is used to apply all migrations in order
# It can be run manually or as part of container initialization

# Determine run mode
if [ -z "$POSTGRES_DB" ]; then
  # Manual mode - script was run from command line
  
  # Verify we are in the right environment
  if [ ! -f "Cargo.toml" ]; then
    echo "Error: This script must be run from the OxiCloud project root directory"
    exit 1
  fi

  # Check if postgres container is running
  POSTGRES_ID=$(docker-compose ps -q postgres 2>/dev/null)
  if [ -z "$POSTGRES_ID" ]; then
    echo "Error: PostgreSQL container is not running. Start it with 'docker-compose up -d postgres'"
    exit 1
  fi
  
  # Set parameters for manual mode
  DB_NAME="oxicloud"
  MIGRATIONS_DIR="./migrations"
  MIGRATIONS_CMD="docker-compose exec -T postgres psql -U postgres"
else
  # Auto mode - script is running inside postgres container during initialization
  echo "Running in PostgreSQL container initialization mode"
  
  # Set parameters for auto mode
  DB_NAME="$POSTGRES_DB"
  MIGRATIONS_DIR="/docker-entrypoint-initdb.d/migrations"
  MIGRATIONS_CMD="psql -U $POSTGRES_USER"
fi

echo "Applying migrations from $MIGRATIONS_DIR to database $DB_NAME"

# Ensure schema is created (fallback)
$MIGRATIONS_CMD -d "$DB_NAME" -c "CREATE SCHEMA IF NOT EXISTS auth;" 2>/dev/null

# Get each SQL file from migrations directory in alphabetical order
for sql_file in $(find $MIGRATIONS_DIR -name "*.sql" | sort); do
  echo "Applying migration: $(basename $sql_file)"
  
  # Run the migration
  if [ -z "$POSTGRES_DB" ]; then
    # Manual mode - run through docker-compose
    docker-compose exec -T postgres psql -U postgres -d "$DB_NAME" -f "/docker-entrypoint-initdb.d/migrations/$(basename $sql_file)"
  else
    # Auto mode - run directly
    psql -U "$POSTGRES_USER" -d "$DB_NAME" -f "$sql_file"
  fi
  
  # Check if migration was successful
  if [ $? -ne 0 ]; then
    echo "Error: Failed to apply migration $sql_file"
    exit 1
  fi
done

echo "All migrations applied successfully"