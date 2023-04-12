use crate::core_args::{CoreArgs, LogLevel};
use clap::Parser;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub fn build_app_state<'a>() -> AppState<'a> {
    let args = CoreArgs::parse();
    let env_file = if args.dev { ".env.dev" } else { ".env" };
    dotenv::from_filename(env_file).ok();
    println!("{:?}", args);
    AppState {
        dev_mode: args.dev,
        log_level: args.log_level,
        app_name: String::from("RUSTY CORE API"),
        cron_time: Duration::from_secs(args.cron_time),
        scheduler_time: std::env::var("RUSTY_SCHEDULER").unwrap_or_else(|_| args.sch_time),
        max_endpoint_count: args.max_endpoint_count,
        arc_map: Arc::new(Mutex::new(HashMap::<&str, CtxRequesterDataPerEndpoint>::new())),
    }
}

#[derive(Debug)]
pub struct CtxRequesterDataPerEndpoint {
    pub endpoint_count_for_ip: u32,
}

impl CtxRequesterDataPerEndpoint {
    pub fn add_endpoint_count(&mut self) {
        self.endpoint_count_for_ip = self.endpoint_count_for_ip + 1;
    }
}

#[derive(Debug)]
pub struct AppState<'a> {
    pub dev_mode: bool,
    pub log_level: LogLevel,
    pub app_name: String,
    pub cron_time: Duration,
    pub scheduler_time: String,
    pub max_endpoint_count: u64,
    pub arc_map: Arc<Mutex<HashMap<&'a str, CtxRequesterDataPerEndpoint>>>,
}

impl<'a> AppState<'a> {
    pub fn update_endpoint_count(&mut self, key: &'a str) {
        let mut map = self.arc_map.lock().unwrap();
        let data_for_endpoint = CtxRequesterDataPerEndpoint {
            endpoint_count_for_ip: 0,
        };
        map.entry(&key).or_insert(data_for_endpoint);
        let data_for_endpoint = CtxRequesterDataPerEndpoint {
            endpoint_count_for_ip: map.get(&key).unwrap().endpoint_count_for_ip + 1,
        };
        map.insert(&key, data_for_endpoint);
    }
    pub fn get_endpoint_count(&self, key: &'a str) -> u32 {
        self.arc_map.lock().unwrap().get(key).unwrap().endpoint_count_for_ip
    }
}
