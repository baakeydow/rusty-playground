//! Retreive data from pocket api => https://getpocket.com/developer/docs/v3/retrieve

use crate::{
    dtkpocket::pocket_utils::get_pocket_consumer_key,
    dtkutils::{
        dtk_reqwest::{get_dtk_response, send_post_request},
        utils::is_rusty_dev,
    },
};

use super::pocket_model::{self, PocketCodeResponseBody, PocketTokenResponseBody};

const REDIRECT_URI: &str = "http://baakey.rusty.com/auth_pocket";
const POCKET_USER_REQUEST_URI: &str = "https://getpocket.com/v3/oauth/request";
const POCKET_AUTH_URI: &str = "https://getpocket.com/v3/oauth/authorize";
const POCKET_RETRIEVE_URI: &str = "https://getpocket.com/v3/get";
const POCKET_PUSH_URI: &str = "https://getpocket.com/v3/send";

fn get_payload_for_pocket_code() -> serde_json::Value {
    serde_json::json!({
        "redirect_uri": &REDIRECT_URI,
        "consumer_key": &get_pocket_consumer_key(),
        "state": "rusty"
    })
}

/// get auth url for user to accepet connection
pub async fn get_user_auth_url() -> Result<(String, String), String> {
    let response = match send_post_request(&POCKET_USER_REQUEST_URI, get_payload_for_pocket_code()).await {
        Ok(r) => get_dtk_response(r).await,
        Err(err) => {
            let err_str = err.to_string();
            println!("Request failed: {}", err_str);
            return Err(err_str);
        }
    };

    let raw_json = &response.res.unwrap();
    let pocket_body_response: PocketCodeResponseBody = serde_json::from_value(raw_json.clone()).unwrap();

    let user_auth_url = format!(
        "https://getpocket.com/auth/authorize?request_token={token}&redirect_uri={redirect_uri}",
        token = pocket_body_response.code,
        redirect_uri = if !is_rusty_dev() {
            &REDIRECT_URI
        } else {
            "http://localhost:4242/auth_pocket"
        }
    );

    Ok((user_auth_url, pocket_body_response.code))
}

/// get pocket user access_token
pub async fn get_access_token(code: &str) -> Result<PocketTokenResponseBody, ()> {
    let payload = serde_json::json!({
        "consumer_key": &get_pocket_consumer_key(),
        "code": code
    });
    let response = match send_post_request(&POCKET_AUTH_URI, payload).await {
        Ok(r) => get_dtk_response(r).await,
        Err(err) => {
            let err_str = err.to_string();
            println!("Request failed: {}", err_str);
            return Err(());
        }
    };
    println!("Response: {:#?}", response.res);
    let raw_json = &response.res.expect("Code is not valid !");
    let pocket_body_response: PocketTokenResponseBody = serde_json::from_value(raw_json.clone()).unwrap();

    Ok(pocket_body_response)
}

/// puch new data to pocket
pub struct PocketPushData {
    /// url to push
    pub url: String,
    /// title of the url
    pub title: String,
    /// tags to add to the url
    pub tags: String,
}
/// puch new data to pocket
pub async fn push_pocket_data(
    access_token: &str,
    data_to_push: Vec<PocketPushData>,
) -> Result<pocket_model::PocketSendResponse, ()> {
    let actions = data_to_push
        .iter()
        .map(|data| {
            serde_json::json!({
                "action": "add",
                "url": &data.url,
                "title": &data.title,
                "tags": &data.tags,
            })
        })
        .collect::<Vec<serde_json::Value>>();
    let payload = serde_json::json!({
        "consumer_key": &get_pocket_consumer_key(),
        "access_token": &access_token,
        "actions": actions,
    });
    let response = match send_post_request(&POCKET_PUSH_URI, payload).await {
        Ok(r) => get_dtk_response(r).await,
        Err(err) => {
            let err_str = err.to_string();
            println!("Request failed: {}", err_str);
            return Err(());
        }
    };
    let raw_json = &response.res.unwrap();
    let pocket_body_response: pocket_model::PocketSendResponse = serde_json::from_value(raw_json.clone()).expect("wtf");

    Ok(pocket_body_response)
}

/// get pocket user access_token
pub async fn retreive_pocket_data(
    access_token: &str,
    since: Option<i64>,
) -> Result<pocket_model::PocketExtractResponse, ()> {
    let payload = serde_json::json!({
        "consumer_key": &get_pocket_consumer_key(),
        "access_token": &access_token,
        "since": since,
        "state": "all",
        "sort": "newest",
        "detailType": "complete"
    });
    let response = match send_post_request(&POCKET_RETRIEVE_URI, payload).await {
        Ok(r) => get_dtk_response(r).await,
        Err(err) => {
            let err_str = err.to_string();
            println!("Request failed: {}", err_str);
            return Err(());
        }
    };
    let raw_json = &response.res.unwrap();
    let pocket_body_response: pocket_model::PocketExtractResponse =
        serde_json::from_value(raw_json.clone()).expect("wtf");

    Ok(pocket_body_response)
}
