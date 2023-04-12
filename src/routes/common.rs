use crate::{
    app_state::AppState,
    toolz::utils::{get_ip_addr, inc_request_count},
};
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use rusty_lib::{
    dtkmongo::dtk_connect::{get_dtkmongo_client, get_mongodb_uri},
    dtkpocket::{
        pocket::{self, save_pocket},
        pocket_model::{DtkPocketData, DtkPocketResponse, PockerUrlResponse, QualifiedPocketData},
        pocket_utils::{get_pocket_collection_name, get_pocket_db_name},
    },
    dtkutils::{
        dtk_reqwest::{get_data_from_body, RequestBodyParser},
        utils::{is_rusty_dev, is_valid_mongo_search},
    },
};
use mongodb::bson::{doc, Document};
use std::{collections::HashSet, sync::Mutex};

pub async fn hey(req: HttpRequest, data: web::Data<Mutex<AppState<'_>>>) -> impl Responder {
    let count = inc_request_count(&req, data);
    let ip = get_ip_addr(&req);
    HttpResponse::Ok().body(format!("Hey {ip} welcome, endpoint requested: {count}"))
}

#[get("/")]
pub async fn hello(req: HttpRequest, data: web::Data<Mutex<AppState<'_>>>) -> impl Responder {
    let count = inc_request_count(&req, data);
    let ip = get_ip_addr(&req);
    HttpResponse::Ok().body(format!(
        "Hey {ip} welcome to RUSTY CORE API, endpoint requested: {count}"
    ))
}

#[post("/echo")]
pub async fn echo((req, req_body, data): (HttpRequest, String, web::Data<Mutex<AppState<'_>>>)) -> impl Responder {
    let count = inc_request_count(&req, data);
    let ip = get_ip_addr(&req);
    log::info!("Hey {ip}, endpoint requested: {count}");
    HttpResponse::Ok().body(req_body)
}

pub async fn get_current_user(
    (req, req_body, _data): (HttpRequest, String, web::Data<Mutex<AppState<'_>>>),
) -> impl Responder {
    let dtk_user_body = get_data_from_body(req_body);
    let client = get_dtkmongo_client(get_mongodb_uri().as_str()).await;
    let coll = client
        .database(&get_pocket_db_name())
        .collection::<Document>("pocket_users");
    let filter = doc! {
        "user_email": dtk_user_body.email.clone(),
    };
    let user_doc = coll.find_one(filter, None).await.unwrap();
    if user_doc.is_some() {
        let ip = get_ip_addr(&req);
        log::info!("Welcome {ip} => {:#?}", dtk_user_body);
        HttpResponse::Ok().json(dtk_user_body)
    } else {
        HttpResponse::Forbidden().finish()
    }
}

pub async fn delete_current_user(
    (_req, req_body, _data): (HttpRequest, String, web::Data<Mutex<AppState<'_>>>),
) -> impl Responder {
    let dtk_user_body = get_data_from_body(req_body);
    let client = get_dtkmongo_client(get_mongodb_uri().as_str()).await;
    let user_coll = client
        .database(&get_pocket_db_name())
        .collection::<Document>("pocket_users");
    let user_filter = doc! {
        "user_id": dtk_user_body.id.clone(),
    };
    user_coll.delete_one(user_filter, None).await.unwrap();
    let pocket_db_name = get_pocket_db_name();
    let pocket_coll_name = get_pocket_collection_name();
    let pocket_coll = client
        .database(&pocket_db_name)
        .collection::<DtkPocketData>(&pocket_coll_name);
    let pocket_filter = doc! {
        "user_id": dtk_user_body.id.clone(),
    };
    pocket_coll.delete_many(pocket_filter, None).await.unwrap();
    HttpResponse::Ok().finish()
}

pub async fn get_pocket_url(
    (_req, req_body, _data): (HttpRequest, String, web::Data<Mutex<AppState<'_>>>),
) -> impl Responder {
    let dtk_user_body = get_data_from_body(req_body);
    let client = get_dtkmongo_client(get_mongodb_uri().as_str()).await;
    let user_coll = client
        .database(&get_pocket_db_name())
        .collection::<Document>("pocket_users");
    let user_filter = doc! {
        "user_id": dtk_user_body.id.clone(),
    };
    let user = user_coll.find_one(user_filter, None).await.unwrap();
    let url_code = rusty_lib::dtkpocket::pocket_auth::get_user_auth_url().await;
    HttpResponse::Ok().json(PockerUrlResponse {
        url: url_code.as_ref().unwrap().0.clone(),
        code: url_code.unwrap().1,
        is_connected: user.is_some(),
    })
}

pub async fn connect_token(
    (_req, req_body, _data): (HttpRequest, String, web::Data<Mutex<AppState<'_>>>),
) -> impl Responder {
    let payload = serde_json::from_str::<RequestBodyParser>(&req_body).unwrap();
    let pocket_body_res = rusty_lib::dtkpocket::pocket_auth::get_access_token(&payload.code.clone().unwrap()).await;
    println!("{:#?}", pocket_body_res);
    if !pocket_body_res.is_ok() {
        return HttpResponse::BadRequest().body("Invalid code");
    } else {
        let dtk_user_body = get_data_from_body(req_body);
        let client = get_dtkmongo_client(get_mongodb_uri().as_str()).await;
        let coll = client
            .database(&get_pocket_db_name())
            .collection::<Document>("pocket_users");
        let filter = doc! {
            "user_email": dtk_user_body.email.clone(),
        };
        let user_doc = coll.find_one(filter, None).await.unwrap();
        if user_doc.is_none() {
            let access_token = pocket_body_res.as_ref().unwrap().access_token.clone();
            let user_name = pocket_body_res.as_ref().unwrap().username.clone();
            let user_doc = doc! {
                "user_id": dtk_user_body.id.clone(),
                "user_email": dtk_user_body.email,
                "user_name": dtk_user_body.name,
                "user_lvl": dtk_user_body.lvl,
                "user_token": dtk_user_body.token,
                "pocket_code": payload.code.unwrap(),
                "pocket_token": access_token.clone(),
                "pocket_user_name": user_name,
            };
            coll.insert_one(user_doc, None).await.unwrap();
            save_pocket(dtk_user_body.id.to_string(), access_token, None).await;
        } else {
            return HttpResponse::BadRequest().body("Pocket already connected");
        }
    }
    HttpResponse::Ok().json(pocket_body_res.unwrap())
}

fn get_pocket_filters(
    id: Option<String>,
    src_type: Option<String>,
    filter_tags: Vec<String>,
    filter_search: Option<String>,
) -> mongodb::bson::Document {
    let mut filters = doc! {};

    if filter_search.is_some()
        && filter_search.as_ref().unwrap().len() > 0
        && is_valid_mongo_search(filter_search.as_ref().unwrap())
    {
        filters.insert("$text", doc! { "$search": filter_search.unwrap() });
    }
    if id.is_some() {
        filters.insert("user_id", id.as_ref().unwrap());
    }
    if src_type.is_some() {
        filters.insert("src_type", src_type.unwrap());
    }
    if filter_tags.len() > 0 {
        filters.insert("tags", doc! {"$in": filter_tags});
    }

    filters
}

pub async fn get_public_pocket(
    (_req, req_body, _data): (HttpRequest, String, web::Data<Mutex<AppState<'_>>>),
) -> impl Responder {
    let payload = get_data_from_body(req_body);
    let client = get_dtkmongo_client(get_mongodb_uri().as_str()).await;
    let db_name = if is_rusty_dev() {
        "baakey_dev_rusty"
    } else {
        "baakey_prod_rusty"
    };
    let user_coll = client.database(db_name).collection::<Document>("users");

    let root_user = user_coll
        .find_one(
            mongodb::bson::doc! {
                "email": "baakey@rusty.com",
            },
            None,
        )
        .await
        .unwrap();

    let root_user_id = root_user
        .unwrap()
        .get("_id")
        .unwrap()
        .as_object_id()
        .unwrap()
        .to_string();

    log::info!("[Payload from body] => {:#?}", payload);

    let all = pocket::get_pocket_data(
        get_pocket_filters(Some(root_user_id), None, [].to_vec(), Some(payload.filter_search)),
        false,
    )
    .await;

    let dtk_pocket_data: Vec<QualifiedPocketData> = all
        .clone()
        .into_iter()
        .map(|elem| QualifiedPocketData::from(elem))
        .collect();

    HttpResponse::Ok().json(DtkPocketResponse {
        qualified: dtk_pocket_data
            .into_iter()
            .filter(|item| !item.rusty_pocket_item.tags.contains(&"private".to_string()))
            .collect(),
        unique_tags: [].to_vec(),
        instagram: [].to_vec(),
        twitter: [].to_vec(),
    })
}

pub async fn get_private_pocket(
    (_req, req_body, _data): (HttpRequest, String, web::Data<Mutex<AppState<'_>>>),
) -> impl Responder {
    let payload = get_data_from_body(req_body);

    log::info!("[Payload from body] => {:#?}", payload);

    let id = payload.id;
    let filter_tags = payload.filter_tags.clone();
    let filter_search = payload.filter_search.clone();

    let without_filters =
        pocket::get_pocket_data(get_pocket_filters(Some(id.clone()), None, [].to_vec(), None), false).await;

    let all = pocket::get_pocket_data(
        get_pocket_filters(Some(id.clone()), None, filter_tags.clone(), Some(filter_search.clone())),
        false,
    )
    .await;

    let instagram_tags = if filter_tags.clone().contains(&"instagram".to_string()) {
        filter_tags.clone()
    } else {
        filter_tags
            .clone()
            .into_iter()
            .chain(["instagram".to_string()].to_vec().into_iter())
            .collect()
    };
    let instagram = pocket::get_pocket_data(
        get_pocket_filters(
            Some(id.clone()),
            Some("instagram".to_string()),
            instagram_tags,
            Some(filter_search.clone()),
        ),
        false,
    )
    .await;
    let twitter_tags = if filter_tags.clone().contains(&"twitter".to_string()) {
        filter_tags.clone()
    } else {
        filter_tags
            .clone()
            .into_iter()
            .chain(["twitter".to_string()].to_vec().into_iter())
            .collect()
    };
    let twitter = pocket::get_pocket_data(
        get_pocket_filters(
            Some(id.clone()),
            Some("twitter".to_string()),
            twitter_tags,
            Some(filter_search),
        ),
        true,
    )
    .await;

    let mut unique_tags: HashSet<String> = HashSet::new();
    for item in without_filters.iter().cloned() {
        unique_tags.extend(item.tags);
    }
    let mut sorted_tags = unique_tags.into_iter().collect::<Vec<String>>();
    sorted_tags.sort_by(|a, b| a.cmp(b));

    let qualified_pocket_data: Vec<QualifiedPocketData> = all
        .clone()
        .into_iter()
        .map(|elem| QualifiedPocketData::from(elem))
        .collect();

    HttpResponse::Ok().json(DtkPocketResponse {
        qualified: qualified_pocket_data,
        unique_tags: sorted_tags,
        instagram,
        twitter,
    })
}
