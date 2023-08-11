use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderModel {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub customer_name: String,
    pub product_name: String,
}

#[allow(non_snake_case)]
#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct CustomerModel {
    pub customer_id: sqlx::types::Uuid,
    pub customer_name: Option<String>,
    pub customer_surname: Option<String>,
}

