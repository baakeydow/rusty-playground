//! Mongodb driver

use mongodb::{options::ClientOptions, Client};

/// Get a mongodb client
pub async fn get_dtkmongo_client(conn_str: &str) -> Client {
    // Parse a connection string into an options struct.
    let mut client_options = ClientOptions::parse(conn_str).await.unwrap();

    // Manually set an option.
    client_options.app_name = Some("core-rusty-api".to_string());

    // Get a handle to the deployment.
    let client = Client::with_options(client_options).expect("failed to connect");

    client
}

/// Get mongodb URI
pub fn get_mongodb_uri() -> String {
    std::env::var("RUSTY_MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into())
}

/// Get mongodb db name
pub fn get_mongodb_main_db() -> String {
    std::env::var("RUSTY_MAIN_DB").unwrap_or_else(|_| "baakey_dev_rusty".into())
}

/// Get db names
pub async fn get_db_names(client: &Client) -> Vec<String> {
    let db_names = client.list_database_names(None, None).await.unwrap();
    db_names
}

/// Get collection names from db
pub async fn get_collection_names(client: &Client, db_name: &str) -> Vec<String> {
    let db = client.database(db_name);
    let coll_names = db.list_collection_names(None).await.unwrap();
    coll_names
}


// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// async fn list_deployment() {
//     println!("Mongo Connect");
//     let client = get_dtkmongo_client(get_mongodb_uri().as_str()).await;

//     // List the names of the databases in that deployment.
//     if let Ok(db_names) = client.list_database_names(None, None).await {
//         for db_name in db_names {
//             println!("\n # {:#?}", db_name);
//             let collection_names = get_collection_names(&client, &db_name).await;
//             for coll_name in collection_names {
//                 println!("- {}", coll_name);
//             }
//         }
//     };
// }
