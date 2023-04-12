use crate::app_state::AppState;
use actix_web::web;
use actix_web::HttpRequest;
use chrono::Local;
use rusty_lib::dtkutils::utils::format_datetime;
use rusty_lib::dtkutils::utils::log_env_vars;
use std::sync::Mutex;

pub fn display_cron_debug(shared_data: &web::Data<Mutex<AppState<'static>>>) {
    log::debug!("[RUSTY_CRON]: => =========================>");
    log::debug!("[RUSTY_CRON]: => {}", format_datetime(Local::now()));
    if let Ok(mut_r_data) = &mut shared_data.lock() {
        if mut_r_data.dev_mode {
            log::debug!("[RUSTY_CRON]: => {:#?}", mut_r_data);
        }
        for (key, value) in &*mut_r_data.arc_map.lock().unwrap() {
            log::debug!("[RUSTY_CRON]: {} => {:?}", key, value);
        }
    };
    log::debug!("[RUSTY_CRON]: => =========================>");
    log::debug!("-------------------------------------------");
}

pub fn setup_core_env(shared_data: &web::Data<Mutex<AppState<'static>>>) {
    let log_level = shared_data.lock().unwrap().log_level.to_string();
    let dev_mode = shared_data.lock().unwrap().dev_mode;
    std::env::set_var("RUST_LOG", "actix_web=info");
    std::env::set_var("RUSTY_DEV_MODE", dev_mode.to_string());
    env_logger::init_from_env(
        env_logger::Env::new()
            .filter_or("RUSTY_LOG_LEVEL", format!("{log_level}").to_lowercase())
            .write_style_or("RUSTY_LOG_STYLE", "always"),
    );
    log_env_vars();
}

pub fn inc_request_count(req: &HttpRequest, data: web::Data<Mutex<AppState<'_>>>) -> u32 {
    let mut state = data.lock().unwrap();
    let ip = get_ip_addr(req);
    let request_id = req.path();
    let key = Box::leak(format!("{request_id}-{ip}").into_boxed_str());
    state.update_endpoint_count(key);
    let count = state.get_endpoint_count(key);
    count
}

pub fn get_ip_addr(req: &HttpRequest) -> String {
    let ip = match req.connection_info().realip_remote_addr() {
        Some(val) => val.to_string(),
        None => panic!("Failed to retrieve ip"),
    };
    ip
}

