#!/usr/bin/env bash
set -e

cd "${0%/*}" || exit # cd script directory
PGPASSWORD=postgres psql -h 127.0.0.1 -U postgres -c "DROP DATABASE IF EXISTS dv_report";
PGPASSWORD=postgres psql -h 127.0.0.1 -U postgres -tc "SELECT 1 FROM pg_user WHERE usename = 'dv_report'" | grep -q 1 ||  PGPASSWORD=postgres psql -h 127.0.0.1 -U postgres -c "CREATE USER dv_report WITH ENCRYPTED PASSWORD 'dv_report';"
PGPASSWORD=postgres psql -h 127.0.0.1 -U postgres -c "CREATE DATABASE dv_report;"
PGPASSWORD=postgres psql -h 127.0.0.1 -U postgres -c "GRANT ALL ON DATABASE dv_report TO dv_report;"
PGPASSWORD=postgres psql -h 127.0.0.1 -U postgres -c "ALTER DATABASE dv_report OWNER TO dv_report;"
cd ../_migrations || exit
DATABASE_URL=postgres://dv_report:dv_report@127.0.0.1/dv_report sqlx migrate run
