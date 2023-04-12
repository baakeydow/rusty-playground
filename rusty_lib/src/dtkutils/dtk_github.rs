//! Retreive data from github API
#![allow(missing_docs)]

use crate::dtkmongo::dtk_connect::get_dtkmongo_client;
use crate::dtkmongo::dtk_connect::get_mongodb_uri;
use crate::dtkutils::dtk_reqwest::send_get_request;
use crate::dtkutils::dtk_reqwest::validate_response;
use chrono::Duration;
use chrono::Utc;
use futures::StreamExt;
use mongodb::bson;
use mongodb::bson::Document;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
/// Dummy struct for deserializing json
pub struct GithubData {}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Github starred repo
pub struct StarredRepo {
    pub id: i64,
    pub name: Option<String>,
    pub full_name: Option<String>,
    pub html_url: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub git_url: Option<String>,
    pub ssh_url: Option<String>,
    pub clone_url: Option<String>,
    pub homepage: Option<String>,
    pub stargazers_count: Option<i64>,
    pub language: Option<String>,
    pub open_issues_count: Option<i64>,
    pub topics: Option<Vec<String>>,
    pub forks: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub pushed_at: Option<String>,
}

/// Get all repository stargazers
pub async fn get_all_stargazers(owner: &str, repo: &str) -> Result<Vec<GithubData>, reqwest::Error> {
    let mut users = Vec::new();
    let mut page = 1;
    loop {
        let url = format!(
            "https://api.github.com/repos/{owner}/{repo}/stargazers?page={}&per_page=100",
            page
        );
        let response = send_get_request(&url).await;
        if response.is_err() {
            println!("{:#?}", response.err().unwrap());
            return Ok(users);
        }
        let response = response.unwrap();
        if validate_response(&response).is_none() {
            println!("Response is not valid: {:#?}", response.status());
            return Ok(users);
        }
        let response = response.json::<Vec<GithubData>>().await;
        if response.is_err() {
            println!("{:#?}", response.err().unwrap());
            return Ok(users);
        }
        let response = response.unwrap();
        if response.is_empty() {
            return Ok(users);
        }
        users.extend(response);
        page += 1;
    }
}

/// Get all starred repositories for a user
pub async fn get_starred(owner: &str, mut page: i32, recurse: bool) -> Result<Vec<StarredRepo>, reqwest::Error> {
    let mut starred = Vec::new();
    loop {
        let url = format!("https://api.github.com/users/{owner}/starred?page={page}&per_page=100");
        let response = send_get_request(&url).await;
        if response.is_err() {
            println!("{:#?}", response.err().unwrap());
            return Ok(starred);
        }
        let response = response.unwrap();
        if validate_response(&response).is_none() {
            println!("Response is not valid: {:#?}", response.status());
            return Ok(starred);
        }
        let response = response.json::<Vec<StarredRepo>>().await;
        if response.is_err() {
            println!("{:#?}", response.err().unwrap());
            return Ok(starred);
        }
        let response = response.unwrap();
        if response.is_empty() {
            return Ok(starred);
        }
        starred.extend(response);
        if !recurse {
            return Ok(starred);
        }
        page += 1;
    }
}

/// Save all starred repositories for baakeydow
pub async fn save_all_starred() {
    let github_db_name = "rusty-github".to_string();
    let github_coll_name = "baakeydow".to_string();
    let client = get_dtkmongo_client(get_mongodb_uri().as_str()).await;
    let github_coll = client
        .database(&github_db_name)
        .collection::<Document>(&github_coll_name);
    let github_data = get_starred("baakeydow", 1, false).await;
    let res = github_data.unwrap();
    for data in res {
        let filter = mongodb::bson::doc! { "id": data.id };
        let mut doc = mongodb::bson::to_document(&data).unwrap();
        let now = Utc::now();
        if github_coll.find(filter.to_owned(), None).await.unwrap().next().await.is_some() {
            doc.insert("updated_at", bson::to_bson(&now).unwrap());
        } else {
            doc.insert("saved_at", bson::to_bson(&now).unwrap());
        }
        let update = mongodb::bson::doc! { "$set": doc };
        let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
        github_coll
            .update_one(filter, update, Some(options))
            .await
            .expect("Failed to insert github data");
    }

}

/// Get all starred repositories for baakeydow
pub async fn retreive_github_data() -> Vec<StarredRepo> {
    let github_db_name = "rusty-github".to_string();
    let github_coll_name = "baakeydow".to_string();
    let client = get_dtkmongo_client(get_mongodb_uri().as_str()).await;
    let github_coll = client
        .database(&github_db_name)
        .collection::<Document>(&github_coll_name);
    let minutes_ago = Utc::now() - Duration::minutes(60);
    let filter = mongodb::bson::doc! {
        "saved_at": { "$gte": minutes_ago.to_rfc3339() }
    };
    let cursor = github_coll.find(filter, None).await.unwrap();
    let github_data: Vec<StarredRepo> = cursor
                .map(|res| mongodb::bson::from_bson(mongodb::bson::Bson::Document(res.unwrap())).unwrap())
                .collect::<Vec<StarredRepo>>()
                .await;
    github_data
}