#!/usr/bin/env bash
set -e

echo "*********** START MIGRATION ***********"
# create submerge user if not exists
psql -U postgres -tc "SELECT 1 FROM pg_user WHERE usename = 'submerge'" | grep -q 1 || psql -U postgres -c "CREATE USER submerge WITH ENCRYPTED PASSWORD 'submerge';"
# create the crystal database if not exists
psql -U postgres -tc "SELECT 1 FROM pg_database WHERE datname = 'submerge_crystal'" | grep -q 1 || psql -U postgres -c "CREATE DATABASE submerge_crystal;"
psql -U postgres -c "ALTER DATABASE submerge_crystal OWNER TO submerge;"
# apply migrations
MIGRATION_FILES_DIR="/submerge/migrations"
for file in "$MIGRATION_FILES_DIR"/*.up.sql; do
    psql -U submerge -d submerge_crystal -f "${file}"
done
echo "************ END MIGRATION ************"