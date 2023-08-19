#!/bin/bash
cp local.env .env
docker compose -f docker-compose-local.yaml up --force-recreate -V -d

sleep 5 

cargo sqlx prepare --database-url "postgresql://postgres:postgres@localhost:5432/postgres"

cargo test

rm .env

docker compose -f docker-compose-local.yaml down
