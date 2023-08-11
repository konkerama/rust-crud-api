use crate::{
    mongo::MONGO,
    pg::PG,
    response::{
        CustomerListResponse, DeleteOrderResponse, GenericResponse, OrderListResponse,
        SingleCustomerResponse, SingleOrderResponse,
    },
    schema::{CreateCustomerSchema, CreateOrderSchema, FilterOptions},
    Result,
};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

pub async fn health_checker_handler() -> Result<impl IntoResponse> {
    const MESSAGE: &str = "Build CRUD API with Rust and MongoDB";
    tracing::info!("{}", MESSAGE);
    let response_json = &GenericResponse {
        status: "success".to_string(),
        message: MESSAGE.to_string(),
    };
    Ok((StatusCode::OK, Json(serde_json::json!(response_json))))
}

// POST /api/pg
#[axum_macros::debug_handler]
pub async fn create_customer_handler(
    State(db): State<PG>,
    Json(body): Json<CreateCustomerSchema>,
) -> Result<impl IntoResponse> {
    let result = db.create_customer(&body).await?;

    Ok((StatusCode::CREATED, Json(result)))
}

pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::FORBIDDEN, "nothing to see here")
}

// GET /api/pg
pub async fn list_customer_handler(
    opts: Option<Query<FilterOptions>>,
    State(db): State<PG>,
) -> Result<Json<CustomerListResponse>> {
    let Query(opts) = opts.unwrap_or_default();
    let limit = opts.limit.unwrap_or(10) as i64;
    let offset = opts.page.unwrap_or(0) as i64;
    let result = db.list_customers(limit, offset).await?;

    Ok(Json(result.unwrap()))
}

// GET /api/pg/<customer-name>
pub async fn get_customer_handler(
    id: Path<String>,
    State(db): State<PG>,
) -> Result<Json<SingleCustomerResponse>> {
    let result = db.get_customer(&id).await?;

    Ok(Json(result.unwrap()))
}

// DELETE /api/pg/<customer-name>
pub async fn delete_customer_handler(
    id: Path<String>,
    State(db): State<PG>,
) -> Result<Json<SingleCustomerResponse>> {
    let result = db.delete_customer(&id).await?;

    Ok(Json(result.unwrap()))
}

// UPDATE /api/pg/<customer-name>
pub async fn update_customer_handler(
    id: Path<String>,
    State(db): State<PG>,
    Json(body): Json<CreateCustomerSchema>,
) -> Result<Json<SingleCustomerResponse>> {
    let result = db.update_customer(&id, &body).await?;

    Ok(Json(result))
}

// POST /api/mongo
pub async fn create_order_handler(
    State(mongo): State<MONGO>,
    Json(body): Json<CreateOrderSchema>,
) -> Result<Json<SingleOrderResponse>> {
    let result = mongo.create_order(&body).await?;

    Ok(Json(result))
}

// GET /api/mongo
pub async fn list_order_handler(
    opts: Option<Query<FilterOptions>>,
    State(mongo): State<MONGO>,
) -> Result<Json<OrderListResponse>> {
    let Query(opts) = opts.unwrap_or_default();
    let limit = opts.limit.unwrap_or(10) as i64;
    let offset = opts.page.unwrap_or(1) as i64;
    let result = mongo.fetch_orders(limit, offset).await?;

    Ok(Json(result))
}

// GET /api/mongo/:id
pub async fn get_order_handler(
    id: Path<String>,
    State(mongo): State<MONGO>,
) -> Result<Json<SingleOrderResponse>> {
    let result = mongo.get_order(&id).await?;

    Ok(Json(result))
}

// PATCH /api/mongo/:id
pub async fn update_order_handler(
    id: Path<String>,
    State(mongo): State<MONGO>,
    Json(body): Json<CreateOrderSchema>,
) -> Result<Json<SingleOrderResponse>> {
    let result = mongo.edit_order(&id, &body).await?;

    Ok(Json(result))
}

// DELETE /api/mongo/:id
pub async fn delete_order_handler(
    id: Path<String>,
    State(mongo): State<MONGO>,
) -> Result<Json<DeleteOrderResponse>> {
    let result = mongo.delete_order(&id).await?;

    Ok(Json(result))
}
