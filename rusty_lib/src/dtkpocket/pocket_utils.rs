//! Pocket utils

use crate::{
    dtkmongo::dtk_connect::{self, get_dtkmongo_client, get_mongodb_uri},
    dtkpocket::pocket_model::PocketData,
    dtkutils::{dtk_github::retreive_github_data, utils::remove_duplicate_hashmap},
};
use mongodb::{
    bson::{doc, Document},
    options::IndexOptions,
    Client, IndexModel,
};
use serde_json::Value;
use std::collections::HashMap;

use super::{
    pocket_auth::{push_pocket_data, PocketPushData},
    pocket_model::{DtkPocketData, PocketSrcType},
};

/// import github stars to pocket
pub async fn import_github_stars() {
    let data = retreive_github_data().await;
    let client = get_dtkmongo_client(get_mongodb_uri().as_str()).await;
    let db_name = get_pocket_db_name();
    let pocket_users_coll = client.database(&db_name).collection::<Document>("pocket_users");
    let root_user = pocket_users_coll
        .find_one(
            mongodb::bson::doc! {
                "user_email": "baakey@rusty.com",
            },
            None,
        )
        .await
        .unwrap();
    let root_user_token = root_user
        .unwrap()
        .get("pocket_token")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let push_data: Vec<PocketPushData> = data
        .into_iter()
        .map(|repo| {
            let url = repo.html_url.unwrap_or_else(|| repo.url.unwrap());
            let title = repo.name.unwrap_or_else(|| repo.full_name.unwrap());
            let tags = repo.language.unwrap_or_else(|| "".to_string());
            PocketPushData { url, title, tags }
        })
        .collect();
    if push_data.len() > 0 {
        log::info!("Pushing {} data to pocket", push_data.len());
        let _res = push_pocket_data(&root_user_token, push_data).await;
    }
}

/// Get pocket Since
pub async fn get_pocket_since() -> Option<i64> {
    let client = get_dtkmongo_client(get_mongodb_uri().as_str()).await;
    let db_name = get_pocket_db_name();
    let coll_name = get_pocket_collection_name();
    if pocket_collection_exist(&client, &db_name, &coll_name).await {
        Some(chrono::Utc::now().timestamp() - 12 * 60 * 60)
    } else {
        None
    }
}

/// Get pocket source type
pub fn get_pocket_src_type(pocket_item: PocketData) -> String {
    let url = get_valid_url(pocket_item);
    match url.as_str() {
        x if x.contains("instagram.com") => PocketSrcType::INSTAGRAM.to_string().into(),
        x if x.contains("twitter.com") => PocketSrcType::TWITTER.to_string().into(),
        x if x.contains("youtube.com") || x.contains("youtu.be") => PocketSrcType::YOUTUBE.to_string().into(),
        x if x.contains("github.com") || x.contains("github1s.com") => PocketSrcType::GITHUB.to_string().into(),
        _ => PocketSrcType::ARTICLE.to_string().into(),
    }
}

/// Check if pocket collection exist
pub async fn pocket_collection_exist(client: &mongodb::Client, db_name: &str, coll_name: &str) -> bool {
    let all_collection = dtk_connect::get_collection_names(&client, &db_name).await;
    all_collection.contains(&coll_name.to_string())
}

/// Get pocket consumer key
pub fn get_pocket_consumer_key() -> String {
    std::env::var("RUSTY_POCKET_CONSUMER_KEY").unwrap_or_else(|_| "asdf".into())
}

/// Get mongodb db name
pub fn get_pocket_db_name() -> String {
    std::env::var("RUSTY_POCKET_DB").unwrap_or_else(|_| "rusty_pocket".into())
}

/// Get mongodb collection name
pub fn get_pocket_collection_name() -> String {
    std::env::var("RUSTY_POCKET_COLL").unwrap_or_else(|_| "pocket_data".into())
}

/// Get valid pocket url
pub fn get_valid_url(pocket_item: PocketData) -> String {
    if pocket_item.given_url.is_empty() {
        pocket_item.resolved_url
    } else {
        pocket_item.given_url
    }
}

/// Get valid pocket title
pub fn get_valid_title(pocket_item: PocketData) -> String {
    if pocket_item.given_title.is_empty() {
        pocket_item.resolved_title
    } else {
        pocket_item.given_title
    }
}

/// Creates an index on the "item_id" field to force the values to be unique and "url" field to be text
pub async fn create_pocket_coll_indexes(client: &Client, coll_name: &str) {
    println!("# => Creating pocket indexes for {}", coll_name);
    let model_item = IndexModel::builder()
        .keys(doc! { "item_id": 1 })
        .options(IndexOptions::builder().unique(true).build())
        .build();
    let model_text = IndexModel::builder()
        .keys(doc! { "url": "text", "title": "text", "excerpt": "text" })
        .options(IndexOptions::builder().unique(false).build())
        .build();
    client
        .database(get_pocket_db_name().as_str())
        .collection::<PocketData>(coll_name)
        .create_indexes(vec![model_item, model_text], None)
        .await
        .expect("creating indexes should succeed");
}

/// Get default article tag from url
pub fn get_default_article_tag(url: &str) -> String {
    match url {
        x if x.contains("instagram.com") => "instagram".to_string(),
        x if x.contains("twitter.com") => "twitter".to_string(),
        x if x.contains("youtube.com") || x.contains("youtu.be") => "youtube".to_string(),
        x if x.contains("github.com") || x.contains("github1s.com") => "github".to_string(),
        _ => "article".to_string(),
    }
}

/// Inject tag from pocket url
pub fn inject_tags_from_url(pocket_item: PocketData) -> Vec<String> {
    let url = if pocket_item.given_url.is_empty() {
        pocket_item.resolved_url
    } else {
        pocket_item.given_url
    };
    let mut tags = pocket_tags_to_vec(pocket_item.tags.clone()).unwrap_or_default();
    let platform_tags = vec![
        ("youtube.com", "youtube"),
        ("youtu.be", "youtube"),
        ("github.com", "github"),
        ("github1s.com", "github"),
        ("twitter.com", "twitter"),
        ("instagram.com", "instagram"),
    ];
    for (platform, tag) in platform_tags {
        if url.contains(platform) && !tags.contains(&tag.to_string()) {
            tags.push(tag.to_string());
        }
    }
    if tags.contains(&"github".to_string()) {
        tags.push("tech".to_string());
    }
    if tags.is_empty() {
        tags.push("article".to_string());
    }
    tags
}

/// Format pocket tags to vec
pub fn pocket_tags_to_vec(option: Option<Value>) -> Option<Vec<String>> {
    if !option.is_some() {
        return None;
    }
    let mut result = Vec::new();
    for (_key, value) in option.unwrap().as_object().unwrap() {
        let tag = value["tag"].as_str().unwrap();
        result.push(tag.to_lowercase().to_owned());
    }
    Some(result)
}

/// Format PocketData into DtkPocketData
pub fn format_pocket_data(pocket_data: HashMap<String, PocketData>, user_id: &str) -> HashMap<String, DtkPocketData> {
    let mut dtk_pocket_data: HashMap<String, DtkPocketData> = HashMap::new();
    for (item_id, pocket_item) in pocket_data {
        let dtk_pocket_item = DtkPocketData::from_other_type(pocket_item, user_id);
        dtk_pocket_data.insert(item_id, dtk_pocket_item);
    }
    remove_duplicate_hashmap(&mut dtk_pocket_data, "url");
    dtk_pocket_data
}

/// Save pocket data to mongodb
pub async fn update_pocket_data(
    client: &Client,
    db_name: &str,
    coll_name: &str,
    user_id: &str,
    pocket_data: HashMap<String, PocketData>,
) {
    if !pocket_collection_exist(client, db_name, coll_name).await {
        create_pocket_coll_indexes(client, coll_name).await;
    }
    let coll = client.database(db_name).collection::<DtkPocketData>(coll_name);

    let dtk_pocket_data = format_pocket_data(pocket_data, user_id);

    for (item_id, pocket_item) in dtk_pocket_data {
        log::info!(
            "# => Saving pocket data for item_id: {} => [{}] => ({})",
            item_id,
            pocket_item.title,
            pocket_item.url
        );
        let filter = doc! { "item_id": item_id, "url": &pocket_item.url };
        let update = doc! { "$set": mongodb::bson::to_document(&pocket_item).unwrap() };
        let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
        coll.update_one(filter, update, Some(options))
            .await
            .expect("update should succeed");
    }
}
