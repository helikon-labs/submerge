#!/usr/bin/env bash
set -euxo pipefail

cd "${0%/*}" || exit # cd script directory
PGPASSWORD=postgres psql -h 127.0.0.1 -U postgres -c "DROP DATABASE IF EXISTS submerge_crystal_test";
PGPASSWORD=postgres psql -h 127.0.0.1 -U postgres -tc "SELECT 1 FROM pg_user WHERE usename = 'submerge'" | grep -q 1 ||  PGPASSWORD=postgres psql -h 127.0.0.1 -U postgres -c "CREATE USER submerge WITH ENCRYPTED PASSWORD 'submerge';"
PGPASSWORD=postgres psql -h 127.0.0.1 -U postgres -c "CREATE DATABASE submerge_crystal_test;"
PGPASSWORD=postgres psql -h 127.0.0.1 -U postgres -c "GRANT ALL ON DATABASE submerge_crystal_test TO submerge;"
PGPASSWORD=postgres psql -h 127.0.0.1 -U postgres -c "ALTER DATABASE submerge_crystal_test OWNER TO submerge;"
cd ../_migrations/crystal || exit
DATABASE_URL=postgres://submerge:submerge@127.0.0.1/submerge_crystal_test sqlx migrate run