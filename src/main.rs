mod error;
mod handler;
mod helper;
mod model;
mod mongo;
mod pg;
mod response;
mod route;
mod schema;

pub use self::error::{Error, Result};

use autometrics::prometheus_exporter;
// use dotenvy::dotenv;
use helper::Config;
use mongo::MONGO;
use pg::PG;
use route::create_router;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, Registry};

#[tokio::main]
async fn main() {
    prometheus_exporter::init();
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "app=info,tower_http=trace");
    }
    if std::env::var_os("LOG_LEVEL").is_none() {
        std::env::set_var("LOG_LEVEL", "info");
    }
    let env_log_level = std::env::var("LOG_LEVEL").unwrap();

    if let Some(level_filter) = string_to_level_filter(&env_log_level) {
        let subscriber = Registry::default()
            .with(level_filter)
            .with(tracing_subscriber::fmt::Layer::default().with_writer(std::io::stdout));

        tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
    } else {
        eprintln!("Invalid log level: {}", env_log_level);
    }

    tracing::info!("Initializing config...");
    let config = Config::init();

    tracing::info!("Retrieving Configuration Variables ...");
    let pg_username: String = config.get_config("POSTGRES_USER");
    let pg_passwd: String = config.get_config("POSTGRES_PASSWORD");
    let pg_url: String = config.get_config("POSTGRES_URL");
    let pg_db: String = config.get_config("POSTGRES_URL");
    let mongodb_username: String = config.get_config("ME_CONFIG_MONGODB_ADMINUSERNAME");
    let mongodb_passwd: String = config.get_config("ME_CONFIG_MONGODB_ADMINPASSWORD");
    let mongodb_server: String = config.get_config("ME_CONFIG_MONGODB_SERVER");

    tracing::info!("Setting up connection to Postgresql...");
    let pg = PG::init(pg_username, pg_passwd, pg_url, pg_db)
        .await
        .unwrap();
    tracing::info!("Setting up connection to MongoDB...");
    let mongo = MONGO::init(mongodb_username, mongodb_passwd, mongodb_server)
        .await
        .unwrap();

    let app = create_router(pg.clone(), mongo.clone());

    tracing::info!("🚀 Server started successfully");
    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn string_to_level_filter(level: &String) -> Option<LevelFilter> {
    match level.to_lowercase().as_str() {
        "error" => Some(LevelFilter::ERROR),
        "warn" => Some(LevelFilter::WARN),
        "info" => Some(LevelFilter::INFO),
        "debug" => Some(LevelFilter::DEBUG),
        "trace" => Some(LevelFilter::TRACE),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{response::*, schema::*};
    use axum::http::Method;
    use axum::{
        body::Body,
        http::{self, Request, StatusCode},
        Router,
    };
    use serde_json::json;
    use tower::ServiceExt; // for `oneshot` and `ready`

    async fn init() -> Router {
        let config = Config::init();

        // retrieve configuration variables
        let pg_username: String = config.get_config("POSTGRES_USER");
        let pg_passwd: String = config.get_config("POSTGRES_PASSWORD");
        let pg_url: String = config.get_config("POSTGRES_URL");
        let pg_db: String = config.get_config("POSTGRES_DB");
        let mongodb_username: String = config.get_config("ME_CONFIG_MONGODB_ADMINUSERNAME");
        let mongodb_passwd: String = config.get_config("ME_CONFIG_MONGODB_ADMINPASSWORD");
        let mongodb_server: String = config.get_config("ME_CONFIG_MONGODB_SERVER");

        let pg = PG::init(pg_username, pg_passwd, pg_url, pg_db)
            .await
            .unwrap();
        let mongo = MONGO::init(mongodb_username, mongodb_passwd, mongodb_server)
            .await
            .unwrap();

        create_router(pg.clone(), mongo.clone())
    }

    fn get_customer_model(name: &str, surname: &str) -> CreateCustomerSchema {
        CreateCustomerSchema {
            customer_name: name.to_string(),
            customer_surname: surname.to_string(),
        }
    }

    fn get_order_schema(customer_name: &str, product_name: &str) -> CreateOrderSchema {
        CreateOrderSchema {
            customer_name: customer_name.to_string(),
            product_name: product_name.to_string(),
        }
    }

    async fn api_call(method: Method, uri: &str, body: Body) -> (StatusCode, serde_json::Value) {
        // dotenv().expect(".env file not found");
        let app = init().await;
        let response = app
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();

        let response_body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        (status, serde_json::from_slice(&response_body).unwrap())
    }
    #[tokio::test]
    async fn health_check() {
        let (status_code, response) =
            api_call(http::Method::GET, "/api/healthchecker", Body::empty()).await;
        assert_eq!(status_code, StatusCode::OK);
        let expected_json = GenericResponse {
            status: "success".to_string(),
            message: "Build CRUD API with Rust and MongoDB".to_string(),
        };

        assert_eq!(
            response.get("message").unwrap().as_str().unwrap(),
            expected_json.message
        );
    }

    #[tokio::test]
    async fn create_customer() {
        let input = get_customer_model("paul", "doe");

        let (status_code, response) = api_call(
            http::Method::POST,
            "/api/pg",
            Body::from(serde_json::to_vec(&json!(input)).unwrap()),
        )
        .await;
        println!("{:?}", response);
        assert_eq!(status_code, StatusCode::CREATED);
        assert_eq!(
            response.get("name").unwrap().as_str().unwrap(),
            input.customer_name.as_str()
        );
        assert_eq!(
            response.get("surname").unwrap().as_str().unwrap(),
            input.customer_surname.as_str()
        );
    }

    #[tokio::test]
    async fn get_customer() {
        let input = get_customer_model("Blanche", "Jarvis");

        let (status_code, response) = api_call(
            http::Method::POST,
            "/api/pg",
            Body::from(serde_json::to_vec(&json!(input)).unwrap()),
        )
        .await;
        println!("{:?}", response);
        assert_eq!(status_code, StatusCode::CREATED);

        let id = response.get("id").unwrap().as_str().unwrap();
        let uri = format!("/api/pg/{}", id);

        let (status_code, response_get) = api_call(http::Method::GET, &uri, Body::empty()).await;
        println!("{:?}", response_get);
        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(
            response_get.get("name").unwrap().as_str().unwrap(),
            input.customer_name.as_str()
        );
        assert_eq!(
            response_get.get("surname").unwrap().as_str().unwrap(),
            input.customer_surname.as_str()
        );
    }

    #[tokio::test]
    async fn list_customers() {
        let input = get_customer_model("Rafael", "Scott");

        let (status_code, response) = api_call(
            http::Method::POST,
            "/api/pg",
            Body::from(serde_json::to_vec(&json!(input)).unwrap()),
        )
        .await;
        println!("{:?}", response);
        assert_eq!(status_code, StatusCode::CREATED);

        let (status_code, response) = api_call(http::Method::GET, "/api/pg", Body::empty()).await;
        println!("{:?}", response);
        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(response.get("status").unwrap().as_str().unwrap(), "success");
    }

    #[tokio::test]
    async fn delete_customer() {
        let input = get_customer_model("Polly", "Shepard");

        let (status_code, response) = api_call(
            http::Method::POST,
            "/api/pg",
            Body::from(serde_json::to_vec(&json!(input)).unwrap()),
        )
        .await;
        println!("{:?}", response);
        assert_eq!(status_code, StatusCode::CREATED);

        let id = response.get("id").unwrap().as_str().unwrap();
        let uri = format!("/api/pg/{}", id);

        let (status_code, response_get) = api_call(http::Method::DELETE, &uri, Body::empty()).await;
        println!("{:?}", response_get);
        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(
            response_get.get("status").unwrap().as_str().unwrap(),
            "deleted"
        );
        assert_eq!(response_get.get("id").unwrap().as_str().unwrap(), id);
    }

    #[tokio::test]
    async fn patch_customer() {
        let original_input = get_customer_model("Polly", "Shepard");
        let modified_input = get_customer_model("Hattie", "Rodgers");

        let (status_code, response_create) = api_call(
            http::Method::POST,
            "/api/pg",
            Body::from(serde_json::to_vec(&json!(original_input)).unwrap()),
        )
        .await;
        println!("{:?}", response_create);
        assert_eq!(status_code, StatusCode::CREATED);

        let id = response_create.get("id").unwrap().as_str().unwrap();
        let uri = format!("/api/pg/{}", id);

        let (status_code, response_patch) = api_call(
            http::Method::PATCH,
            &uri,
            Body::from(serde_json::to_vec(&json!(modified_input)).unwrap()),
        )
        .await;
        println!("{:?}", response_patch);
        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(
            response_patch.get("status").unwrap().as_str().unwrap(),
            "success"
        );

        let (status_code, response_get) = api_call(http::Method::GET, &uri, Body::empty()).await;
        println!("{:?}", response_get);
        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(
            response_get.get("name").unwrap().as_str().unwrap(),
            modified_input.customer_name.as_str()
        );
        assert_eq!(
            response_get.get("surname").unwrap().as_str().unwrap(),
            modified_input.customer_surname.as_str()
        );
    }

    #[tokio::test]
    async fn create_order() {
        let input = get_order_schema("paul", "banana");

        let (status_code, response) = api_call(
            http::Method::POST,
            "/api/mongo",
            Body::from(serde_json::to_vec(&json!(input)).unwrap()),
        )
        .await;
        println!("{:?}", response);
        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(
            response["data"]["order"]["customer_name"].as_str().unwrap(),
            input.customer_name.as_str()
        );
        assert_eq!(
            response["data"]["order"]["product_name"].as_str().unwrap(),
            input.product_name.as_str()
        );
    }

    #[tokio::test]
    async fn get_order() {
        let input = get_order_schema("paul", "banana");

        let (status_code, response_post) = api_call(
            http::Method::POST,
            "/api/mongo",
            Body::from(serde_json::to_vec(&json!(input)).unwrap()),
        )
        .await;
        println!("{:?}", response_post);
        assert_eq!(status_code, StatusCode::OK);

        let id = response_post["data"]["order"]["id"].as_str().unwrap();
        let uri = format!("/api/mongo/{}", id);

        let (status_code, response_get) = api_call(http::Method::GET, &uri, Body::empty()).await;
        println!("{:?}", response_get);
        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(
            response_get["data"]["order"]["customer_name"]
                .as_str()
                .unwrap(),
            input.customer_name.as_str()
        );
        assert_eq!(
            response_get["data"]["order"]["product_name"]
                .as_str()
                .unwrap(),
            input.product_name.as_str()
        );
    }

    #[tokio::test]
    async fn list_orders() {
        let input = get_order_schema("paul", "banana");

        let (status_code, response_post) = api_call(
            http::Method::POST,
            "/api/mongo",
            Body::from(serde_json::to_vec(&json!(input)).unwrap()),
        )
        .await;
        println!("{:?}", response_post);
        assert_eq!(status_code, StatusCode::OK);

        let (status_code, response_get) =
            api_call(http::Method::GET, "/api/mongo", Body::empty()).await;
        println!("{:?}", response_get);
        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(
            response_get.get("status").unwrap().as_str().unwrap(),
            "success"
        );
    }

    #[tokio::test]
    async fn delete_order() {
        let input = get_order_schema("paul", "banana");

        let (status_code, response_post) = api_call(
            http::Method::POST,
            "/api/mongo",
            Body::from(serde_json::to_vec(&json!(input)).unwrap()),
        )
        .await;
        println!("{:?}", response_post);
        assert_eq!(status_code, StatusCode::OK);

        let id = response_post["data"]["order"]["id"].as_str().unwrap();
        let uri = format!("/api/mongo/{}", id);

        let (status_code, response_delete) =
            api_call(http::Method::DELETE, &uri, Body::empty()).await;
        println!("{:?}", response_delete);
        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(response_delete["status"].as_str().unwrap(), "deleted");
        assert_eq!(response_delete["id"].as_str().unwrap(), id);
    }

    #[tokio::test]
    async fn patch_order() {
        let original_input = get_order_schema("paul", "banana");
        let modified_input = get_order_schema("mark", "apple");

        let (status_code, response_post) = api_call(
            http::Method::POST,
            "/api/mongo",
            Body::from(serde_json::to_vec(&json!(original_input)).unwrap()),
        )
        .await;
        println!("{:?}", response_post);
        assert_eq!(status_code, StatusCode::OK);

        let id = response_post["data"]["order"]["id"].as_str().unwrap();
        let uri = format!("/api/mongo/{}", id);

        let (status_code, response_patch) = api_call(
            http::Method::PATCH,
            &uri,
            Body::from(serde_json::to_vec(&json!(modified_input)).unwrap()),
        )
        .await;
        println!("{:?}", response_patch);
        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(
            response_patch.get("status").unwrap().as_str().unwrap(),
            "success"
        );

        let (status_code, response_get) = api_call(http::Method::GET, &uri, Body::empty()).await;
        println!("{:?}", response_get);
        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(
            response_get["data"]["order"]["customer_name"]
                .as_str()
                .unwrap(),
            modified_input.customer_name.as_str()
        );
        assert_eq!(
            response_get["data"]["order"]["product_name"]
                .as_str()
                .unwrap(),
            modified_input.product_name.as_str()
        );
    }
}
