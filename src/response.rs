use serde::Serialize;

#[derive(Serialize)]
pub struct GenericResponse {
    pub status: String,
    pub message: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Debug)]
pub struct OrderResponse {
    pub id: String,
    pub customer_name: String,
    pub product_name: String,
}

#[derive(Serialize, Debug)]
pub struct OrderData {
    pub order: OrderResponse,
}

#[derive(Serialize, Debug)]
pub struct SingleOrderResponse {
    pub status: String,
    pub data: OrderData,
}

#[derive(Serialize, Debug)]
pub struct DeleteOrderResponse {
    pub status: String,
    pub id: String,
}

#[derive(Serialize, Debug)]
pub struct OrderListResponse {
    pub status: String,
    pub results: usize,
    pub orders: Vec<OrderResponse>,
}

#[derive(Serialize, Debug)]
pub struct CustomerResponse {
    pub id: String,
    pub name: String,
    pub surname: String,
}

#[derive(Serialize, Debug)]
pub struct CustomerListResponse {
    pub status: String,
    pub data: Vec<CustomerResponse>,
}

#[derive(Serialize, Debug)]
pub struct SingleCustomerResponse {
    pub status: String,
    pub id: String,
    pub name: String,
    pub surname: String,
}