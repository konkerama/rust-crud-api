use axum::{
    routing::{get, post},
    Router,
};

use crate::Error;
use crate::{handler::*, mongo::MONGO, pg::PG};

use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use axum::response::{IntoResponse, Response};
use axum::{middleware, Json};
use serde_json::json;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub fn create_router(pg: PG, mongo: MONGO) -> Router {
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:8000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    Router::new()
        .route("/api/healthchecker", get(health_checker_handler))
        .nest(
            "/api/pg",
            Router::new()
                .route(
                    "/",
                    post(create_customer_handler).get(list_customer_handler),
                )
                .route(
                    "/:name",
                    get(get_customer_handler)
                        .delete(delete_customer_handler)
                        .patch(update_customer_handler),
                )
                .with_state(pg),
        )
        .nest(
            "/api/mongo",
            Router::new()
                .route("/", post(create_order_handler).get(list_order_handler))
                .route(
                    "/:id",
                    get(get_order_handler)
                        .patch(update_order_handler)
                        .delete(delete_order_handler),
                )
                .with_state(mongo),
        )
        .layer(cors)
        .layer(middleware::map_response(main_response_mapper))
        .layer(TraceLayer::new_for_http())
        .fallback(handler_404)
}

#[allow(unused_variables)]
async fn main_response_mapper(req_method: Method, res: Response) -> Response {
    tracing::info!("->> {:<12} - main_response_mapper", "RES_MAPPER");

    // -- Get the eventual response error.
    let service_error = res.extensions().get::<Error>();
    let client_status_error = service_error.map(|se| se.client_status_and_error());

    // -- If client error, build the new response.
    let error_response = client_status_error
        .as_ref()
        .map(|(status_code, client_error)| {
            let client_error_body = json!({
                "error": {
                    "type": client_error.as_ref(),
                }
            });

            tracing::error!("    ->> client_error_body: {client_error_body}");

            // Build the new response from the client_error_body
            (*status_code, Json(client_error_body)).into_response()
        });

    // Build and log the server log line.
    // let client_error = client_status_error.unzip().1;
    // tracing::error!("Method: {:?}, client error: {:?}", req_method, client_error);
    // TODO: Need to handler if log_request fail (but should not fail request)

    error_response.unwrap_or(res)
}
