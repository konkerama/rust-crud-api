mod mongo;
mod error;
mod handler;
mod model;
mod response;
mod schema;
mod pg;
mod route;

pub use self::error::{Error, Result};

use mongo::MONGO;
use pg::PG;
use route::create_router;
use tracing_subscriber::{layer::SubscriberExt, Registry};
use tracing::level_filters::LevelFilter;
use tracing::Level;


#[tokio::main]
async fn main() {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "app=info,tower_http=trace");
    }
    let pg = PG::init().await.unwrap();
    let mongo = MONGO::init().await.unwrap();

    // todo remove this
    // dotenv().ok();

    let subscriber = Registry::default()
        .with(LevelFilter::from_level(Level::DEBUG))
        .with(tracing_subscriber::fmt::Layer::default().with_writer(std::io::stdout));

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
    let app = create_router(pg.clone(), mongo.clone());

    tracing::info!("🚀 Server started successfully");
    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}


#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        Router,
        http::{self, Request, StatusCode},
    };
    use serde_json::json;
    use tower::ServiceExt; // for `oneshot` and `ready`
    use crate::{
        response::*,schema::*
    };
    use axum::http::Method;

    async fn init() -> Router{
        let pg = PG::init().await.unwrap();
        let mongo = MONGO::init().await.unwrap();

        create_router(pg.clone(), mongo.clone())
    }

    fn get_customer_model(name: &str, surname: &str) -> CreateCustomerSchema{
        CreateCustomerSchema{
            customer_name: name.to_string(),
            customer_surname: surname.to_string(),
        }
    }

    fn get_order_schema(customer_name: &str, product_name: &str) -> CreateOrderSchema{
        CreateOrderSchema{
            customer_name: customer_name.to_string(),
            product_name: product_name.to_string(),
        }
    }

    async fn api_call(method: Method, uri: &str, body: Body) -> (StatusCode, serde_json::Value) {
        let app = init().await;
        let response = app
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap())
            .await
            .unwrap();
        let status = response.status();

        let response_body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        (status, serde_json::from_slice(&response_body).unwrap())
    }
    #[tokio::test]
    async fn health_check() {
        let (status_code, response)  = api_call(http::Method::GET, "/api/healthchecker", Body::empty()).await;
        assert_eq!(status_code, StatusCode::OK);
        let expected_json = GenericResponse {
            status: "success".to_string(),
            message: "Build CRUD API with Rust and MongoDB".to_string(),
        };

        assert_eq!(response.get("message").unwrap().as_str().unwrap(), expected_json.message);
    }

    #[tokio::test]
    async fn create_customer() {
        let input = get_customer_model("paul", "doe");

        let (status_code, response)  = api_call(http::Method::POST, "/api/pg", Body::from(serde_json::to_vec(&json!(input)).unwrap())).await;
        println!("{:?}", response);
        assert_eq!(status_code, StatusCode::CREATED);
        assert_eq!(response.get("name").unwrap().as_str().unwrap(), input.customer_name.as_str());
        assert_eq!(response.get("surname").unwrap().as_str().unwrap(), input.customer_surname.as_str());
    }

    #[tokio::test]
    async fn get_customer() {
        let input = get_customer_model("Blanche", "Jarvis");

        let (status_code, response)  = api_call(http::Method::POST, "/api/pg", Body::from(serde_json::to_vec(&json!(input)).unwrap())).await;
        println!("{:?}", response);
        assert_eq!(status_code, StatusCode::CREATED);

        let id = response.get("id").unwrap().as_str().unwrap();
        let uri = format!("/api/pg/{}", id);

        let (status_code, response_get)  = api_call(http::Method::GET, &uri, Body::empty()).await;
        println!("{:?}", response_get);
        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(response_get.get("name").unwrap().as_str().unwrap(), input.customer_name.as_str());
        assert_eq!(response_get.get("surname").unwrap().as_str().unwrap(), input.customer_surname.as_str());
    }

    #[tokio::test]
    async fn list_customers() {
        let input = get_customer_model("Rafael", "Scott");

        let (status_code, response)  = api_call(http::Method::POST, "/api/pg", Body::from(serde_json::to_vec(&json!(input)).unwrap())).await;
        println!("{:?}", response);
        assert_eq!(status_code, StatusCode::CREATED);

        let (status_code, response)  = api_call(http::Method::GET, "/api/pg", Body::empty()).await;
        println!("{:?}", response);
        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(response.get("status").unwrap().as_str().unwrap(), "success");
    }

    #[tokio::test]
    async fn delete_customer() {
        let input = get_customer_model("Polly", "Shepard");

        let (status_code, response)  = api_call(http::Method::POST, "/api/pg", Body::from(serde_json::to_vec(&json!(input)).unwrap())).await;
        println!("{:?}", response);
        assert_eq!(status_code, StatusCode::CREATED);

        let id = response.get("id").unwrap().as_str().unwrap();
        let uri = format!("/api/pg/{}", id);

        let (status_code, response_get)  = api_call(http::Method::DELETE, &uri, Body::empty()).await;
        println!("{:?}", response_get);
        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(response_get.get("status").unwrap().as_str().unwrap(), "deleted");
        assert_eq!(response_get.get("id").unwrap().as_str().unwrap(), id);
    }

    #[tokio::test]
    async fn patch_customer() {
        let original_input = get_customer_model("Polly", "Shepard");
        let modified_input = get_customer_model("Hattie", "Rodgers");

        let (status_code, response_create)  = api_call(http::Method::POST, "/api/pg", Body::from(serde_json::to_vec(&json!(original_input)).unwrap())).await;
        println!("{:?}", response_create);
        assert_eq!(status_code, StatusCode::CREATED);

        let id = response_create.get("id").unwrap().as_str().unwrap();
        let uri = format!("/api/pg/{}", id);

        let (status_code, response_patch)  = api_call(http::Method::PATCH, &uri, Body::from(serde_json::to_vec(&json!(modified_input)).unwrap())).await;
        println!("{:?}", response_patch);
        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(response_patch.get("status").unwrap().as_str().unwrap(), "success");

        let (status_code, response_get)  = api_call(http::Method::GET, &uri, Body::empty()).await;
        println!("{:?}", response_get);
        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(response_get.get("name").unwrap().as_str().unwrap(), modified_input.customer_name.as_str());
        assert_eq!(response_get.get("surname").unwrap().as_str().unwrap(), modified_input.customer_surname.as_str());
    }


    #[tokio::test]
    async fn create_order() {
        let input = get_order_schema("paul", "banana");

        let (status_code, response)  = api_call(http::Method::POST, "/api/mongo", Body::from(serde_json::to_vec(&json!(input)).unwrap())).await;
        println!("{:?}", response);
        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(response["data"]["order"]["customer_name"].as_str().unwrap(), input.customer_name.as_str());
        assert_eq!(response["data"]["order"]["product_name"].as_str().unwrap(), input.product_name.as_str());
    }

    #[tokio::test]
    async fn get_order() {
        let input = get_order_schema("paul", "banana");

        let (status_code, response_post)  = api_call(http::Method::POST, "/api/mongo", Body::from(serde_json::to_vec(&json!(input)).unwrap())).await;
        println!("{:?}", response_post);
        assert_eq!(status_code, StatusCode::OK);

        let id = response_post["data"]["order"]["id"].as_str().unwrap();
        let uri = format!("/api/mongo/{}", id);

        let (status_code, response_get)  = api_call(http::Method::GET, &uri, Body::empty()).await;
        println!("{:?}", response_get);
        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(response_get["data"]["order"]["customer_name"].as_str().unwrap(), input.customer_name.as_str());
        assert_eq!(response_get["data"]["order"]["product_name"].as_str().unwrap(), input.product_name.as_str());
    }

    #[tokio::test]
    async fn list_orders() {
        let input = get_order_schema("paul", "banana");

        let (status_code, response_post)  = api_call(http::Method::POST, "/api/mongo", Body::from(serde_json::to_vec(&json!(input)).unwrap())).await;
        println!("{:?}", response_post);
        assert_eq!(status_code, StatusCode::OK);

        let (status_code, response_get)  = api_call(http::Method::GET, "/api/mongo", Body::empty()).await;
        println!("{:?}", response_get);
        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(response_get.get("status").unwrap().as_str().unwrap(), "success");
    }

    #[tokio::test]
    async fn delete_order() {
        let input = get_order_schema("paul", "banana");

        let (status_code, response_post)  = api_call(http::Method::POST, "/api/mongo", Body::from(serde_json::to_vec(&json!(input)).unwrap())).await;
        println!("{:?}", response_post);
        assert_eq!(status_code, StatusCode::OK);

        let id = response_post["data"]["order"]["id"].as_str().unwrap();
        let uri = format!("/api/mongo/{}", id);

        let (status_code, response_delete)  = api_call(http::Method::DELETE, &uri, Body::empty()).await;
        println!("{:?}", response_delete);
        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(response_delete["status"].as_str().unwrap(), "deleted");
        assert_eq!(response_delete["id"].as_str().unwrap(), id);
    }

    #[tokio::test]
    async fn patch_order() {
        let original_input = get_order_schema("paul", "banana");
        let modified_input = get_order_schema("mark", "apple");

        let (status_code, response_post)  = api_call(http::Method::POST, "/api/mongo", Body::from(serde_json::to_vec(&json!(original_input)).unwrap())).await;
        println!("{:?}", response_post);
        assert_eq!(status_code, StatusCode::OK);

        let id = response_post["data"]["order"]["id"].as_str().unwrap();
        let uri = format!("/api/mongo/{}", id);

        let (status_code, response_patch)  = api_call(http::Method::PATCH, &uri, Body::from(serde_json::to_vec(&json!(modified_input)).unwrap())).await;
        println!("{:?}", response_patch);
        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(response_patch.get("status").unwrap().as_str().unwrap(), "success");

        let (status_code, response_get)  = api_call(http::Method::GET, &uri, Body::empty()).await;
        println!("{:?}", response_get);
        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(response_get["data"]["order"]["customer_name"].as_str().unwrap(), modified_input.customer_name.as_str());
        assert_eq!(response_get["data"]["order"]["product_name"].as_str().unwrap(), modified_input.product_name.as_str());
    }

}