//! RUSTY utils for reqwest
extern crate jsonwebtoken as jwt;
use html2md::{Handle, NodeData, StructuredPrinter, TagHandler, TagHandlerFactory};
use http::{HeaderMap, HeaderValue};
use jwt::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use crate::dtkchat::chat_model::{DtkChat, DtkChatUser, DtkChatMessage};

use super::dtk_error::DtkError;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
/// Token info
pub struct TokenInfo {
    /// User id
    pub id: String,
    /// User lvl
    pub lvl: Vec<String>,
    /// User email
    pub email: String,
    /// User name
    pub name: String,
    /// Token iat
    pub iat: i64,
    /// Token exp
    pub exp: i64,
    /// Token aud
    pub aud: String,
    /// Token iss
    pub iss: String,
    /// Token sub
    pub sub: String,
}

#[derive(Serialize, Deserialize)]
/// token request body
pub struct RequestBodyParser {
    /// RUSTY user
    pub user: Option<serde_json::Value>,
    /// Pocket code
    pub code: Option<String>,
    /// Pocket tags
    pub filter_tags: Option<Vec<String>>,
    /// Pocket search
    pub filter_search: Option<String>,
    /// Chat payload
    pub chat_payload: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
/// Pocket User
pub struct DtkRequestBody {
    /// User id
    pub id: String,
    /// User email
    pub email: String,
    /// User name
    pub name: String,
    /// User level
    pub lvl: Vec<String>,
    /// User token
    pub token: String,
    /// Filter tags
    pub filter_tags: Vec<String>,
    /// Filter search
    pub filter_search: String,
    /// Chat payload
    pub chat_payload: DtkChat,
}

/// Get user data pauload
pub fn get_data_from_body(req_body: String) -> DtkRequestBody {
    let payload = serde_json::from_str::<RequestBodyParser>(&req_body).unwrap();
    let user = payload.user.unwrap_or_default();
    let chat_payload = payload.chat_payload.clone().unwrap_or_default();
    DtkRequestBody {
        id: user["id"].as_str().unwrap_or_default().to_string(),
        email: user["email"].as_str().unwrap_or_default().to_string(),
        name: user["name"].as_str().unwrap_or_default().to_string(),
        lvl: if user["lvl"].is_array() {
            user["lvl"]
                .as_array()
                .unwrap()
                .to_vec()
                .iter()
                .map(|x| x.as_str().unwrap().to_string())
                .collect::<Vec<String>>()
        } else {
            vec![]
        },
        token: user["token"].as_str().unwrap_or_default().to_string(),
        filter_tags: payload
            .filter_tags
            .unwrap_or_default()
            .to_vec()
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        filter_search: payload.filter_search.unwrap_or_default(),
        chat_payload: if payload.chat_payload.is_some() {
            DtkChat {
                last_update: chrono::Utc::now().to_string(),
                channel_id: chat_payload["channel_id"].as_str().unwrap().to_string(),
                users: chat_payload["users"]
                    .as_array()
                    .unwrap()
                    .to_vec()
                    .iter()
                    .map(|x| DtkChatUser {
                        id: x["id"].as_str().unwrap().to_string(),
                        name: x["name"].as_str().unwrap().to_string(),
                        email: x["email"].as_str().unwrap().to_string(),
                    })
                    .collect::<Vec<DtkChatUser>>(),
                messages: chat_payload["messages"]
                    .as_array()
                    .unwrap()
                    .to_vec()
                    .iter()
                    .map(|x| DtkChatMessage {
                        sender_id: x["sender_id"].as_str().unwrap().to_string(),
                        date: x["date"].as_str().unwrap().to_string(),
                        message: x["message"].as_str().unwrap().to_string(),
                    })
                    .collect::<Vec<DtkChatMessage>>(),
            }
        } else {
            DtkChat {
                channel_id: "".to_string(),
                last_update: chrono::Utc::now().to_string(),
                users: vec![],
                messages: vec![],
            }
        },
    }
}

/// Get token info
pub fn get_token_info(token: String, user_id: String) -> Result<TokenInfo, DtkError> {
    let mut dir = std::env::current_dir().unwrap();
    dir.push("src/jwtRS256.key.pub");
    let pem_file = dir.to_str().unwrap();
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[user_id]);
    validation.set_required_spec_claims(&["exp", "iat"]);
    validation.set_issuer(&["rusty"]);
    validation.sub = Some("RUSTY".to_string());
    validation.validate_exp = false;
    let mut public_key = Vec::new();
    File::open(pem_file)
        .and_then(|mut f| f.read_to_end(&mut public_key))
        .unwrap();
    let public_key = DecodingKey::from_rsa_pem(&public_key).unwrap();
    let data = match decode::<serde_json::Value>(&token.to_string(), &public_key, &validation) {
        Ok(data) => data,
        Err(err) => return Err(DtkError::from(err.to_string().as_str())),
    };
    Ok(serde_json::from_value::<TokenInfo>(data.claims).unwrap())
}

/// Validate response from reqwest
pub fn validate_response(response: &reqwest::Response) -> Option<bool> {
    match response.status() {
        reqwest::StatusCode::OK => Some(true),
        reqwest::StatusCode::FORBIDDEN => None,
        _ => None,
    }
}

/// Send get reqwest
pub async fn send_get_request(url: &str) -> Result<reqwest::Response, reqwest::Error> {
    let client = reqwest::Client::new();
    client.get(url).header("User-agent", "Reqwest").send().await
}

/// Send post reqwest
pub async fn send_post_request(url: &str, body: serde_json::Value) -> Result<reqwest::Response, reqwest::Error> {
    let client = reqwest::Client::new();
    println!("# => send_post_request info: {} => {:#?}", url, &body);
    client
        .post(url)
        .header("X-Accept", "application/json")
        .header("Content-Type", "application/json; charset=UTF-8")
        .header("User-agent", "Reqwest")
        .json(&body)
        .send()
        .await
}

/// Get html from url
pub async fn get_html(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::get(url).await?;
    // let mut html = String::new();
    let crazy = &response.bytes().await.unwrap();
    let answer = std::str::from_utf8(crazy).unwrap();
    // response.unwrap().read_to_string(&mut html)?;
    Ok(answer.to_string())
}

/// Get html to markdown
pub async fn get_html_to_md(url: &str) -> String {
    // Define a custom tag handler that removes style tags
    struct RemoveUnwantedTagHandler {}
    impl TagHandler for RemoveUnwantedTagHandler {
        fn handle(&mut self, tag: &Handle, printer: &mut StructuredPrinter) {
            match tag.data {
                NodeData::Element { .. } => {
                    printer.append_str("");
                }
                _ => return,
            }
        }

        fn after_handle(&mut self, printer: &mut StructuredPrinter) {
            printer.append_str("");
        }

        fn skip_descendants(&self) -> bool {
            return true;
        }
    }

    // Define a factory for creating RemoveUnwantedTagHandler instances
    struct RemoveTagHandlerFactory {}
    impl TagHandlerFactory for RemoveTagHandlerFactory {
        fn instantiate(&self) -> Box<dyn TagHandler> {
            return Box::new(RemoveUnwantedTagHandler {});
        }
    }
    let html = get_html(url).await.unwrap();
    let mut tag_factory: HashMap<String, Box<dyn TagHandlerFactory>> = HashMap::new();
    tag_factory.insert(String::from("style"), Box::new(RemoveTagHandlerFactory {}));
    tag_factory.insert(String::from("script"), Box::new(RemoveTagHandlerFactory {}));
    tag_factory.insert(String::from("img"), Box::new(RemoveTagHandlerFactory {}));
    let markdown = html2md::parse_html_custom(&html, &tag_factory);
    markdown
}

#[derive(Debug)]
/// Rusty response
pub struct DtkResonse {
    /// ip of called service
    pub addr: std::net::SocketAddr,
    /// headers from reqwestk
    pub headers: HeaderMap<HeaderValue>,
    /// json body response
    pub res: Result<serde_json::Value, reqwest::Error>,
}

/// parse Response into DtkResponse
pub async fn get_dtk_response(r: reqwest::Response) -> DtkResonse {
    DtkResonse {
        addr: r.remote_addr().clone().unwrap(),
        headers: r.headers().clone(),
        res: r.json::<serde_json::Value>().await,
    }
}

/// test html to md
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn html_to_md() {
    let md = get_html_to_md("https://baakeydow.dtksi.com/md/rust/baakeydow").await;
    assert!(md.len() > 0);
}
