use std::env;
use std::sync::Mutex;

use std::time::Instant;

use crate::jwt_auth::JwtAuth;
use crate::ws_chat;
use actix::*;
use actix_files::NamedFile;
use actix_web::{web, Error, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
use rusty_lib::dtkutils::dtk_reqwest::get_token_info;
use rusty_lib::{
    dtkchat::{
        chat::{create_dtk_chat_message, get_all_chat_users, get_all_dtk_chat_for_user},
        chat_model::{ChatForUsers, DtkChat, DtkChatUser},
    },
    dtkutils::dtk_reqwest::get_data_from_body,
};
use serde::Deserialize;

use crate::{
    app_state::AppState,
    toolz::utils::{get_ip_addr, inc_request_count},
};

#[derive(Debug, Deserialize)]
pub struct AuthenticatedRequest {
    user_id: String,
    channel_id: String,
    token: String,
}

pub async fn chat_ws_index() -> impl Responder {
    let mut dir = env::current_dir().unwrap();
    dir.push("src/static/index.html");
    NamedFile::open_async(dir).await.unwrap()
}

/// Entry point for our websocket route
pub async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<ws_chat::server::ChatServer>>,
    info: web::Query<AuthenticatedRequest>,
) -> Result<HttpResponse, Error> {
    let token = info.token.clone();
    let user_id = info.user_id.clone();
    let channel_id = info.channel_id.clone();
    let auth_data = get_token_info(token.clone(), user_id.clone()).unwrap();
    let jwt = JwtAuth {
        claims: auth_data.clone(),
        token: token.to_string(),
    };
    if jwt.claims.id != user_id {
        HttpResponse::Unauthorized().finish();
    }
    let actor = ws_chat::session::WsChatSession {
        id: 0,
        hb: Instant::now(),
        room: channel_id.clone(),
        name: None,
        addr: srv.get_ref().clone(),
        token: Some(token),
        user_id: Some(user_id.clone()),
        channel_id: Some(channel_id.clone()),
    };
    ws::start(actor, &req, stream)
}

pub async fn get_chat((req, req_body, data): (HttpRequest, String, web::Data<Mutex<AppState<'_>>>)) -> impl Responder {
    println!("{:#?}", req_body);
    let count = inc_request_count(&req, data);
    let ip = get_ip_addr(&req);
    let payload = get_data_from_body(req_body);
    let chat = get_all_dtk_chat_for_user(DtkChatUser {
        id: payload.id,
        name: payload.name,
        email: payload.email,
    })
    .await;
    let users = get_all_chat_users().await;
    HttpResponse::Ok().json(ChatForUsers {
        count,
        id: ip,
        chat,
        users,
    })
}

pub async fn post_chat_message(
    (req, req_body, data): (HttpRequest, String, web::Data<Mutex<AppState<'_>>>),
) -> impl Responder {
    let count = inc_request_count(&req, data);
    let ip = get_ip_addr(&req);
    let payload = get_data_from_body(req_body);
    let channel_id = payload.chat_payload.channel_id;
    create_dtk_chat_message(DtkChat {
        channel_id: channel_id.clone(),
        last_update: chrono::Utc::now().to_string(),
        users: payload.chat_payload.users,
        messages: payload.chat_payload.messages,
    })
    .await;
    let chat = get_all_dtk_chat_for_user(DtkChatUser {
        id: payload.id,
        name: payload.name,
        email: payload.email,
    })
    .await;
    let users = get_all_chat_users().await;
    HttpResponse::Ok().json(ChatForUsers {
        count,
        id: ip,
        chat,
        users,
    })
}
