#!/bin/bash

# Definir variables de conexión por defecto
DB_HOST=${PGHOST:-"localhost"}
DB_PORT=${PGPORT:-"5432"}
DB_USER=${PGUSER:-"postgres"}
DB_PASS=${PGPASSWORD:-"postgres"}
DB_NAME=${PGDATABASE:-"postgres"}

# Intentar usar variables de entorno de OxiCloud si están definidas
if [ -n "$OXICLOUD_DB_CONNECTION" ]; then
    # Parse postgres:// connection string
    if [[ $OXICLOUD_DB_CONNECTION =~ postgres://([^:]+):([^@]+)@([^:]+):([0-9]+)/([^?]+) ]]; then
        DB_USER="${BASH_REMATCH[1]}"
        DB_PASS="${BASH_REMATCH[2]}"
        DB_HOST="${BASH_REMATCH[3]}"
        DB_PORT="${BASH_REMATCH[4]}"
        DB_NAME="${BASH_REMATCH[5]}"
    fi
fi

echo "Applying database migrations..."
echo "Using database: postgres://$DB_USER:***@$DB_HOST:$DB_PORT/$DB_NAME"

# Exportar variable PGPASSWORD para psql
export PGPASSWORD="$DB_PASS"

# Ejecutar el script SQL de migración
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -f fix-userrole.sql

# Comprobar si fue exitoso
if [ $? -eq 0 ]; then
    echo "Migration applied successfully!"
else
    echo "Error applying migration."
    exit 1
fi

echo "Database is now ready for use."