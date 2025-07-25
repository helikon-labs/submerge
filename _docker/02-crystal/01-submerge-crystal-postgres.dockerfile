FROM postgres:17.5
ENV POSTGRES_PASSWORD postgres
ENV POSTGRES_HOST postgres
ENV PGDATA /var/lib/postgresql/data
# copy entry point
COPY ./_docker/01-base/03-submerge-postgres-entrypoint.sh /usr/local/bin/submerge-postgres-entrypoint.sh
RUN chmod +x /usr/local/bin/submerge-postgres-entrypoint.sh
# copy migration files
RUN mkdir -p /submerge/migrations/
COPY ./_migrations/crystal/migrations/*.up.sql /submerge/migrations/
# copy init script
COPY ./_docker/02-crystal/01-submerge-crystal-postgres-init.sh /docker-entrypoint-initdb.d/