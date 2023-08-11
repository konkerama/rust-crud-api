use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Default)]
pub struct FilterOptions {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Deserialize, Debug)]
pub struct ParamOptions {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateOrderSchema {
    pub customer_name: String,
    pub product_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateCustomerSchema {
    pub customer_name: String,
    pub customer_surname: String,
}
