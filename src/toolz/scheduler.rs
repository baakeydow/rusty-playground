use crate::app_state::AppState;
use actix::prelude::*;
use actix_web::web;
use chrono::Local;
use cron::Schedule;
use rusty_lib::dtkpocket::pocket::save_all_pocket;
use rusty_lib::dtkpocket::pocket_utils::import_github_stars;
use rusty_lib::dtkutils::dtk_github::save_all_starred;
use rusty_lib::dtkutils::utils::is_rusty_dev;
use std::sync::Mutex;
use std::{str::FromStr, time::Duration};

/// process main task with AppState in ref_data
/// sec   min   hour   day of month   month   day of week   year
/// *     *     *      *              *       *             *
fn process_main_task(sch: &Scheduler) {
    log::debug!("[SCHEDULER]: => =========================>");
    log::debug!("[SCHEDULER]: => =========================>");
    log::debug!(
        "[SCHEDULER] Task event => {:?} - arc_map keys: {:?}",
        Local::now(),
        sch.ref_data.lock().unwrap().arc_map.lock().unwrap().keys().len()
    );
    let mut_r_data = sch.ref_data.lock().unwrap();
    if mut_r_data.dev_mode == false {
        log::debug!("[RUSTY_CRON]: (BEFORE DELETE) => {:#?}", mut_r_data);
        mut_r_data.arc_map.lock().unwrap().clear();
        log::debug!("[RUSTY_CRON]: ArcMap Cleared !");
    }

    if !is_rusty_dev() {
        actix_web::rt::spawn(async move {
            save_all_pocket().await;
        });
        actix_web::rt::spawn(async move {
            save_all_starred().await;
        });
        actix_web::rt::spawn(async move {
            import_github_stars().await;
        });
    } else {
        log::info!("save_pocket is disabled in dev mode");
    }

    log::debug!("[SCHEDULER]: => =========================>");
    log::debug!("[SCHEDULER]: => =========================>");
    log::debug!("^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^");
}

// Define msg
#[derive(Message)]
#[rtype(result = "Result<bool, std::io::Error>")]
pub struct Ping {
    pub ref_data: web::Data<Mutex<AppState<'static>>>,
}

// Define actor
pub struct Scheduler {
    pub ref_data: web::Data<Mutex<AppState<'static>>>,
}

// send AppState to scheduler context
pub async fn start_scheduler(shared_data: web::Data<Mutex<AppState<'static>>>) {
    let addr = Scheduler {
        ref_data: shared_data.clone(),
    }
    .start();
    let result = addr.send(Ping { ref_data: shared_data }).await;
    match result {
        Ok(res) => log::debug!("[SCHEDULER] Got result: {}", res.unwrap()),
        Err(err) => log::debug!("[SCHEDULER] Got error: {}", err),
    }
}

// Task Event logic
impl Scheduler {
    fn schedule_task(&self, ctx: &mut Context<Self>) {
        process_main_task(&self);
        ctx.run_later(
            duration_until_next(&self.ref_data.lock().unwrap().scheduler_time[..]),
            move |this, ctx| this.schedule_task(ctx),
        );
    }
}

// Provide Actor implementation for our actor
impl Actor for Scheduler {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        log::debug!("[SCHEDULER] Actor is alive");

        ctx.run_later(
            duration_until_next(&self.ref_data.lock().unwrap().scheduler_time[..]),
            move |this, ctx| this.schedule_task(ctx),
        );
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        log::debug!("[SCHEDULER] Actor is stopped");
    }
}

impl<'a> Handler<Ping> for Scheduler {
    type Result = Result<bool, std::io::Error>;

    // Save AppState
    fn handle(&mut self, msg: Ping, ctx: &mut Context<Self>) -> Self::Result {
        self.ref_data = msg.ref_data.clone();
        log::debug!(
            "[SCHEDULER] Message received: {:?} - {:?}",
            msg.ref_data.lock().unwrap().arc_map,
            ctx
        );
        Ok(true)
    }
}

pub fn duration_until_next(cron_expression: &str) -> Duration {
    let cron_schedule = Schedule::from_str(cron_expression).unwrap();
    let now = Local::now();
    let next = cron_schedule.upcoming(Local).next().unwrap();
    let duration_until = next.signed_duration_since(now);
    duration_until.to_std().unwrap()
}
