use crate::app_state::AppState;
use actix::clock::interval;
use actix::spawn;
use actix_web::web;
use std::sync::Mutex;

use super::utils::display_cron_debug;

// main application tick tracker with AppState
pub async fn run_main_cron(shared_data: web::Data<Mutex<AppState<'static>>>) {
    spawn(async move {
        let mut interval = interval(shared_data.lock().unwrap().cron_time);
        loop {
            interval.tick().await;
            display_cron_debug(&shared_data);
        }
    });
}
