# Rust Crud api

Sample implementation of a rust web server using axum that integrates with posgresql and mongo db databases

## API Description

Exposes 2 separate crud implementations on `/api/pg` & `/api/mongo` for the PostgreSQL and MongoDB implementations respectively.
You can list/create/update/delete customers by targeting the `/api/pg` path with attributes of `customer_name` & `customer_surname` in the request body.
The same can be applied for MongoDB on the `/api/mongo` path.

## Deployment

The application is packaged on a container for easy reuse on multiple environments. Liquibase is used for managing the PostgreSQL schema.

## How to use

### Prerquisites

Install rust on your system.

### Test

``` bash
cargo install sqlx-cli
./test.sh
```

### Run API locally

``` bash
docker compose up --build --force-recreate -V
```

`docker compose` performs the following steps:

- builds the container
- creates the mongodb and postgesql containers
- creates and runs the liquibase containers that configures the postgresql schema
- creates the pgadming and mongoexpress containers for easy debugging of the databases.

You can target the api using the following example `curl` commands:

``` bash
# health check
curl http://localhost:8000/api/healthchecker -s | jq

# POST create customer 
curl -X POST http://localhost:8000/api/pg -d '{"customer_name": "john","customer_surname": "doe"}' -H "Content-Type: application/json" -s | jq

# GET customer (replace <id> with your customer id)
curl http://localhost:8000/api/pg/<id> -s | jq

# LIST customers
curl http://localhost:8000/api/pg -s | jq

# DELETE customer (replace <id> with your customer id)
curl -X DELETE http://localhost:8000/api/pg/<id> -s | jq

# PATCH customer (replace <id> with your customer id)
curl -X PATCH http://localhost:8000/api/pg/<id> -d '{"customer_name": "mark","customer_surname": "green"}' -H "Content-Type: application/json" -s | jq

# POST order
curl -X POST http://localhost:8000/api/mongo -d '{"customer_name":"mark", "product_name":"apple"}' -H "Content-Type: application/json" -s | jq

# LIST orders
curl http://localhost:8000/api/mongo -s | jq

# GET order (replace <id> with your order id)
curl http://localhost:8000/api/mongo/<id> -s | jq

# PATCH order (replace <id> with your order id)
curl -X PATCH http://localhost:8000/api/mongo/<id> -d '{"customer_name":"paul", "product_name":"banana"}' -H "Content-Type: application/json" -s | jq

# DELETE order (replace <id> with your order id)
curl -X DELETE http://localhost:8000/api/mongo/<id> -s | jq

```

## Todo

- implement tracing using opentelemetry
- modify dependency loading with dotenv file rather than env vars