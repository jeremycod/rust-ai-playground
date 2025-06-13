use mongodb::{Client as MongoClient, Collection };
use mongodb::bson;
use mongodb::options::ClientOptions;

pub(crate) async fn connect_to_mongodb() -> Collection<bson::Document>{
    // Initialize MongoDB client

    let mongodb_connection_string =
        std::env::var("MONGODB_CONNECTION_STRING")
            .expect("MONGODB_CONNECTION_STRING must be set");
    println!("{}", mongodb_connection_string);
    let options = ClientOptions::parse(mongodb_connection_string)
        .await
        .expect("MongoDB client options parse failed ${mongodb://localhost:27017/}");

    let mongodb_client = MongoClient::with_options(options)
        .expect("MongoDB client creation failed");

    let collection: Collection<bson::Document> = mongodb_client
        .database("knowledgebase")
        .collection("memories");
    println!("connected to mongo");
    collection

}