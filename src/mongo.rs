use crate::response::{
    DeleteOrderResponse, OrderData, OrderListResponse, OrderResponse, SingleOrderResponse,
};
use crate::{model::OrderModel, schema::CreateOrderSchema};
use crate::{Error, Result};
use autometrics::autometrics;
use futures::StreamExt;
use mongodb::bson::{doc, oid::ObjectId, Document};
use mongodb::options::{FindOneAndUpdateOptions, FindOptions, ReturnDocument};
use mongodb::{bson, options::ClientOptions, Client, Collection};
use std::convert::TryFrom;
use std::str::FromStr;
use tracing::instrument;

#[derive(Clone, Debug)]
pub struct MONGO {
    pub note_collection: Collection<OrderModel>,
    pub collection: Collection<Document>,
}

impl MONGO {
    #[instrument]
    #[autometrics]
    pub async fn init(
        mongodb_username: String,
        mongodb_passwd: String,
        mongodb_server: String,
    ) -> Result<Self> {
        // let mongodb_username: String = std::env::var("ME_CONFIG_MONGODB_ADMINUSERNAME")
        //     .expect("ME_CONFIG_MONGODB_ADMINUSERNAME must be set.");
        // let mongodb_passwd: String = std::env::var("ME_CONFIG_MONGODB_ADMINPASSWORD")
        //     .expect("ME_CONFIG_MONGODB_ADMINPASSWORD must be set.");
        // let mongodb_server: String = std::env::var("ME_CONFIG_MONGODB_SERVER")
        //     .expect("ME_CONFIG_MONGODB_SERVER must be set.");
        let mongodb_uri = format!(
            "mongodb://{}:{}@{}/",
            mongodb_username, mongodb_passwd, mongodb_server
        );
        let database_name: String =
            std::env::var("MONGO_INITDB_DATABASE").expect("MONGO_INITDB_DATABASE must be set.");
        let mongodb_note_collection: String =
            std::env::var("MONGODB_NOTE_COLLECTION").expect("MONGODB_NOTE_COLLECTION must be set.");

        let mut client_options = ClientOptions::parse(mongodb_uri)
            .await
            .map_err(|e| Error::MongoParsingError { e: (e.to_string()) })?;

        client_options.app_name = Some(database_name.to_string());

        let client = Client::with_options(client_options)
            .map_err(|e| Error::MongoConnectionError { e: (e.to_string()) })?;
        let database = client.database(database_name.as_str());

        let note_collection = database.collection(mongodb_note_collection.as_str());
        let collection = database.collection::<Document>(mongodb_note_collection.as_str());

        tracing::info!("✅ Database connected successfully");

        Ok(Self {
            note_collection,
            collection,
        })
    }

    #[instrument]
    #[autometrics]
    pub async fn fetch_orders(&self, limit: i64, page: i64) -> Result<OrderListResponse> {
        let find_options = FindOptions::builder()
            .limit(limit)
            .skip(
                u64::try_from((page - 1) * limit)
                    .map_err(|e| Error::MongoParsingError { e: (e.to_string()) })?,
            )
            .build();

        let mut cursor = self
            .note_collection
            .find(None, find_options)
            .await
            .map_err(|e| Error::MongoQueryError { e: (e.to_string()) })?;

        let mut json_result: Vec<OrderResponse> = Vec::new();
        while let Some(doc) = cursor.next().await {
            println!("{:?}", doc);
            // unwrap() is allowed as there is no case where an None type will enter the while loop
            json_result.push(self.doc_to_order(&doc.unwrap()));
        }

        let json_note_list = OrderListResponse {
            status: "success".to_string(),
            results: json_result.len(),
            orders: json_result,
        };

        Ok(json_note_list)
    }

    #[instrument]
    #[autometrics]
    pub async fn create_order(&self, body: &CreateOrderSchema) -> Result<SingleOrderResponse> {
        let customer_name = body.customer_name.to_owned();
        let product_name = body.product_name.to_owned();

        let doc = doc! {"customer_name": customer_name, "product_name": product_name};

        let insert_result = self.collection.insert_one(&doc, None).await.map_err(|e| {
            if e.to_string()
                .contains("E11000 duplicate key error collection")
            {
                tracing::error!("🔥 MongoDuplicateError: {:?}", e);
                std::process::exit(1);
            }
            tracing::error!("🔥 MongoQueryError: {:?}", e);
            std::process::exit(1);
        })?;

        let new_id = insert_result
            .inserted_id
            .as_object_id()
            .expect("issue with new _id");

        let order_doc = self
            .note_collection
            .find_one(doc! {"_id":new_id }, None)
            .await
            .map_err(|e| Error::MongoQueryError { e: (e.to_string()) })?
            .ok_or(Error::MongoError)?;

        let note_response = SingleOrderResponse {
            status: "success".to_string(),
            data: OrderData {
                order: self.doc_to_order(&order_doc),
            },
        };

        Ok(note_response)
    }

    #[instrument]
    #[autometrics]
    pub async fn get_order(&self, id: &str) -> Result<SingleOrderResponse> {
        let oid = ObjectId::from_str(id)
            .map_err(|e| Error::MongoInvalidIDError { e: (e.to_string()) })?;

        let note_doc = self
            .note_collection
            .find_one(doc! {"_id":oid }, None)
            .await
            .map_err(|e| Error::MongoQueryError { e: (e.to_string()) })?
            .ok_or(Error::MongoError)?;

        let note_response = SingleOrderResponse {
            status: "success".to_string(),
            data: OrderData {
                order: self.doc_to_order(&note_doc),
            },
        };

        Ok(note_response)
    }

    #[instrument]
    #[autometrics]
    pub async fn edit_order(
        &self,
        id: &str,
        body: &CreateOrderSchema,
    ) -> Result<SingleOrderResponse> {
        let oid = ObjectId::from_str(id)
            .map_err(|e| Error::MongoInvalidIDError { e: (e.to_string()) })?;
        let query = doc! {
            "_id": oid,
        };

        let find_one_and_update_options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();

        let serialized_data = bson::to_bson(body)
            .map_err(|e| Error::MongoSerializeBsonError { e: (e.to_string()) })?;
        let document = serialized_data
            .as_document()
            .ok_or(Error::MongoSerializeError)?;
        let update = doc! {"$set": document};

        let note_doc = self
            .note_collection
            .find_one_and_update(query, update, find_one_and_update_options)
            .await
            .map_err(|e| Error::MongoQueryError { e: (e.to_string()) })?
            .ok_or(Error::MongoError)?;

        let note_response = SingleOrderResponse {
            status: "success".to_string(),
            data: OrderData {
                order: self.doc_to_order(&note_doc),
            },
        };

        Ok(note_response)
    }

    #[instrument]
    #[autometrics]
    pub async fn delete_order(&self, id: &str) -> Result<DeleteOrderResponse> {
        let oid = ObjectId::from_str(id)
            .map_err(|e| Error::MongoInvalidIDError { e: (e.to_string()) })?;

        let _result = self
            .collection
            .delete_one(doc! {"_id":oid }, None)
            .await
            .map_err(|e| Error::MongoQueryError { e: (e.to_string()) })?;

        let order_response = DeleteOrderResponse {
            status: "deleted".to_string(),
            id: id.to_string(),
        };
        Ok(order_response)
    }

    #[instrument]
    #[autometrics]
    fn doc_to_order(&self, order: &OrderModel) -> OrderResponse {
        let order_response = OrderResponse {
            id: order.id.to_hex(),
            customer_name: order.customer_name.to_owned(),
            product_name: order.product_name.to_owned(),
        };

        order_response
    }
}
