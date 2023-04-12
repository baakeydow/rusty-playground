use actix::Actor;
use actix_cors::Cors;
use actix_web::dev::Service;
use actix_web::guard::fn_guard;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpResponse, HttpServer};
use core_rusty_api::jwt_auth::JwtAuth;
use core_rusty_api::routes::chat::chat_route;
use core_rusty_api::toolz::dtksi_cron::run_main_cron;
use core_rusty_api::toolz::scheduler::start_scheduler;
use core_rusty_api::ws_chat::server;
use core_rusty_api::{app_state::build_app_state, routes::chat, routes::common, toolz::utils::setup_core_env};
use futures_util::future::FutureExt;
use log::debug;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_data = web::Data::new(Mutex::new(build_app_state()));
    setup_core_env(&app_data);
    run_main_cron(app_data.clone()).await;
    start_scheduler(app_data.clone()).await;
    println!("[RUSTY_CORE_API](init) => {:#?}", &app_data);
    // set up applications state
    // keep a count of the number of visitors
    let chat_state = Arc::new(AtomicUsize::new(0));

    // start chat server actor
    let server = server::ChatServer::new(chat_state.clone()).start();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://localhost:1342")
            .allowed_origin_fn(|origin, _req_head| {
                origin.as_bytes().ends_with(b".rust.com") || origin.as_bytes().ends_with(b".rusty.com")
            })
            .supports_credentials()
            .allow_any_header()
            .allow_any_method();
        App::new()
            .app_data(web::Data::from(chat_state.clone()))
            .app_data(web::Data::new(server.clone()))
            .app_data(web::PayloadConfig::default().limit(1024 * 1024 * 1024))
            .app_data(app_data.clone())
            .wrap(Logger::default())
            .wrap(cors)
            .wrap_fn(|req, srv| {
                debug!("Hi from start. You requested: {}", req.path());
                srv.call(req).map(|res| {
                    debug!("Hi from response");
                    res
                })
            })
            // .service(web::resource("/").to(chat_ws_index))
            .service(common::hello)
            .service(common::echo)
            .service(
                web::scope("/chat")
                    .guard(fn_guard(|ctx| JwtAuth::new(ctx).is_ok()))
                    .route("/get", web::post().to(chat::get_chat))
                    .route("/post", web::post().to(chat::post_chat_message)),
            )
            .service(
                web::scope("/pocket")
                    .guard(fn_guard(|ctx| JwtAuth::new(ctx).is_ok()))
                    .route("/connect", web::post().to(common::connect_token))
                    .route("/user", web::post().to(common::get_current_user))
                    .route("/delete", web::post().to(common::delete_current_user))
                    .route("/url", web::post().to(common::get_pocket_url))
                    .route("/private", web::post().to(common::get_private_pocket)),
            )
            .route("/chat/ws", web::get().to(chat_route))
            .route("/pocket/public", web::post().to(common::get_public_pocket))
            .route("/hey", web::get().to(common::hey))
            .default_service(web::route().to(|| HttpResponse::Unauthorized()))
    })
    .bind(("0.0.0.0", 1342))?
    .run()
    .await
}
