Postgres is current running with:

POSTGRES_USER=dev
POSTGRES_PASSWORD=password
POSTGRES_DB=fluxbb

# docker run --name pg-fluxbb -e POSTGRES_USER=dev -e POSTGRES_PASSWORD=password -e POSTGRES_DB=fluxbb -p 5432:5432 -d postgres:latest
PGPASSWORD=password psql -h localhost -U dev -d fluxbb -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
