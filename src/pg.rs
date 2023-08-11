use crate::response::{CustomerResponse, SingleCustomerResponse, CustomerListResponse};
use crate::{
    model::CustomerModel, schema::CreateCustomerSchema,
};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use sqlx::types::Uuid;
use crate::{Error, Result};

#[derive(Clone, Debug)]
pub struct PG {
    pub pool: Pool<Postgres>,
}

impl PG {
    pub async fn init() -> Result<Self> { 
        let pg_username: String = 
            std::env::var("POSTGRES_USER").expect("POSTGRES_USER must be set.");
        let pg_passwd: String = 
            std::env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD must be set.");
        let pg_url: String = 
            std::env::var("POSTGRES_URL").expect("POSTGRES_URL must be set.");
        let pg_db: String = 
            std::env::var("POSTGRES_DB").expect("POSTGRES_DB must be set.");
        let pg_uri = 
            format!("postgresql://{}:{}@{}:5432/{}", pg_username, pg_passwd, pg_url, pg_db);

        let pool = match PgPoolOptions::new()
            .max_connections(10)
            .connect(&pg_uri)
            .await
        {
            Ok(pool) => {
                tracing::info!("✅Connection to the database is successful!");
                pool
            }
            Err(err) => {
                tracing::error!("🔥 Failed to connect to the database: {:?}", err);
                std::process::exit(1);
            }
        };

        Ok(Self {
            pool,
        })
    }

    pub async fn create_customer(&self, body: &CreateCustomerSchema) -> Result<SingleCustomerResponse> {
        let name = body.customer_name.to_owned();
        let surname = body.customer_surname.to_owned();
        
        let query_result = sqlx::query_as!(
            CustomerModel,
            "INSERT INTO customer (customer_name,customer_surname) VALUES ($1, $2) RETURNING *",
            name,
            surname,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e|Error::PGError { e: (e.to_string()) })?;

        let customer_response = SingleCustomerResponse {
            id: query_result.customer_id.to_string(),
            name: query_result.customer_name.unwrap_or("john doe".to_string()),
            surname: query_result.customer_surname.unwrap_or("doe".to_string()),
            status: "success".to_string(),
        };

        Ok(customer_response)
    }

    pub async fn list_customers(&self, limit: i64, offset: i64) -> Result<Option<CustomerListResponse>> {
        let query_result = sqlx::query_as!(
            CustomerModel,
            "SELECT * FROM customer ORDER by customer_name LIMIT $1 OFFSET $2",
            limit as i32,
            offset as i32
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e|Error::PGError { e: (e.to_string()) })?;

        tracing::info!("{:?}", query_result);

        let mut json_result: Vec<CustomerResponse> = Vec::new();
        for customer in query_result {
            json_result.push(self.model_to_result(&customer).unwrap());
        }

        let customer_response = CustomerListResponse {
            status: "success".to_string(),
            data: json_result
        };

        Ok(Some(customer_response))
    }

    pub async fn get_customer(&self, id: &String) -> Result<Option<SingleCustomerResponse>> {
        let customer_id = Uuid::parse_str(id)
            .map_err(|e|Error::SqlxUuid { e: (e.to_string()) })?;

        let query_result = sqlx::query_as!(
            CustomerModel,
            "SELECT * FROM customer WHERE customer_id=$1",
            customer_id,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e|Error::PGError { e: (e.to_string()) })?;

        let customer_response = SingleCustomerResponse {
            id: query_result.customer_id.to_string(),
            name: query_result.customer_name.unwrap_or("john".to_string()),
            surname: query_result.customer_surname.unwrap_or("doe".to_string()),
            status: "success".to_string(),
        };

        Ok(Some(customer_response))
    }

    pub async fn delete_customer(&self, id: &String) -> Result<Option<SingleCustomerResponse>> {
        let customer_id = Uuid::parse_str(id)
            .map_err(|e|Error::SqlxUuid { e: (e.to_string()) })?;

        let customer_info = sqlx::query_as!(
            CustomerModel,
            "SELECT * FROM customer WHERE customer_id=$1",
            customer_id,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e|Error::PGError { e: (e.to_string()) })?;


        sqlx::query_as!(
            CustomerModel,
            "DELETE FROM customer WHERE customer_id=$1",
            customer_id,
        )
        .execute(&self.pool)
        .await
        .map_err(|e|Error::PGError { e: (e.to_string()) })?;
    
        let customer_response = SingleCustomerResponse {
            id: customer_info.customer_id.to_string(),
            name: customer_info.customer_name.unwrap_or("john".to_string()),
            surname: customer_info.customer_surname.unwrap_or("doe".to_string()),
            status: "deleted".to_string(),
        };
        Ok(Some(customer_response))
    }

    pub async fn update_customer(&self, id: &String, body: &CreateCustomerSchema) -> Result<SingleCustomerResponse> {
        let customer_id = Uuid::parse_str(id)
            .map_err(|e|Error::SqlxUuid { e: (e.to_string()) })?;

        let name = body.customer_name.to_owned();
        let surname = body.customer_surname.to_owned();

        // ensure customer exists
        let result = self.get_customer(&id).await?;
        tracing::info!("{:?}", result);

        let query_result = sqlx::query_as!(
            CustomerModel,
            "UPDATE customer SET customer_name=$1,customer_surname=$2 WHERE customer_id=$3 RETURNING *",
            name,
            surname,
            customer_id,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e|Error::PGError { e: (e.to_string()) })?;

        let customer_response = SingleCustomerResponse {
            id: query_result.customer_id.to_string(),
            name: query_result.customer_name.unwrap_or("john doe".to_string()),
            surname: query_result.customer_surname.unwrap_or("doe".to_string()),
            status: "success".to_string(),
        };

        Ok(customer_response)
    }


    
    fn model_to_result(&self, customer: &CustomerModel) -> Result<CustomerResponse> {
        let customer_response = CustomerResponse {
            id: customer.customer_id.to_owned().to_string(),
            name: customer.customer_name.to_owned().unwrap(),
            surname: customer.customer_surname.to_owned().unwrap(),
        };

        Ok(customer_response)
    }

}
