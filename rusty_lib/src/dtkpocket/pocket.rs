//! Mongodb Pocket data operations

use futures::stream::StreamExt;
use mongodb::bson::Document;
use mongodb::options::{AggregateOptions, FindOptions};
use std::collections::HashMap;

use super::pocket_model::*;
use super::pocket_utils::*;
use crate::dtkmongo::dtk_connect::*;
use crate::dtkpocket::pocket_auth;

/// Save user pocket data from some time ago
pub async fn save_pocket(user_id: String, token: String, since: Option<i64>) {
    let client = get_dtkmongo_client(get_mongodb_uri().as_str()).await;
    let db_name = get_pocket_db_name();
    let coll_name = get_pocket_collection_name();
    let pocket_data = pocket_auth::retreive_pocket_data(&token, since).await;
    if let Ok(pocket_list) = serde_json::from_value::<HashMap<String, PocketData>>(pocket_data.unwrap().list) {
        update_pocket_data(&client, &db_name, &coll_name, &user_id, pocket_list).await;
    }
}

/// Save pocket data from all users
pub async fn save_all_pocket() {
    let client = get_dtkmongo_client(get_mongodb_uri().as_str()).await;
    let user_coll = client
        .database(&get_pocket_db_name())
        .collection::<Document>("pocket_users");
    let mut users = user_coll.find(None, None).await.unwrap();
    while let Some(user) = users.next().await {
        let user = user.unwrap();
        let user_id = user.get_str("user_id").unwrap();
        let access_token = user.get_str("pocket_token").unwrap();
        let since = get_pocket_since().await;
        save_pocket(user_id.to_string(), access_token.to_string(), since).await;
    }
}

/// Get pocket data from mongodb
pub async fn get_pocket_data(filters: mongodb::bson::Document, dedup_excerpt: bool) -> Vec<DtkPocketData> {
    let client = get_dtkmongo_client(get_mongodb_uri().as_str()).await;
    let db_name = get_pocket_db_name();
    let coll_name = get_pocket_collection_name();
    let coll = client.database(&db_name).collection::<DtkPocketData>(&coll_name);
    match dedup_excerpt {
        true => {
            let pipeline = vec![
                mongodb::bson::doc! { "$match": filters },
                mongodb::bson::doc! {
                    "$group": {
                        "_id": "$excerpt",
                        "data": {
                            "$first": "$$ROOT"
                        }
                    }
                },
                mongodb::bson::doc! {
                    "$replaceRoot": {
                        "newRoot": "$data"
                    }
                },
                mongodb::bson::doc! {
                    "$sort": {
                        "time_added": -1
                    }
                },
            ];
            let options = AggregateOptions::builder().build();
            log::debug!("pipeline: {:?}, option: {:?}", pipeline, options);
            let cursor = coll.aggregate(pipeline, options).await.unwrap();
            let pocket_data: Vec<DtkPocketData> = cursor
                .map(|res| mongodb::bson::from_bson(mongodb::bson::Bson::Document(res.unwrap())).unwrap())
                .collect::<Vec<DtkPocketData>>()
                .await;
            pocket_data
        }
        _ => {
            let options = FindOptions::builder()
                .sort(mongodb::bson::doc! {"time_added": -1})
                .build();
            log::debug!("filters: {:?}, option: {:?}", filters, options);
            let cursor = coll.find(filters, options).await.unwrap();
            let pocket_data: Vec<DtkPocketData> = cursor.map(|res| res.unwrap()).collect::<Vec<DtkPocketData>>().await;
            pocket_data
        }
    }
}
