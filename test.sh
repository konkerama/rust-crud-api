#!/bin/bash
source local.sh
docker compose -f docker-compose-local.yaml up --force-recreate -V -d

sleep 5

cargo sqlx prepare --database-url "postgresql://postgres:postgres@localhost:5432/postgres"

cargo test

docker compose -f docker-compose-local.yaml down
