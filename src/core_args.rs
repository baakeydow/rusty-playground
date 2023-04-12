use clap::{arg_enum, Parser};

arg_enum! {
    #[derive(PartialEq, Debug)]
    pub enum LogLevel {
        ERROR,
        WARN,
        INFO,
        DEBUG,
        TRACE
    }
}

/// RUSTY CORE Args
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct CoreArgs {
    /// Dev mode
    #[clap(short, long)]
    pub dev: bool,

    /// Main application tick
    #[clap(short, long, default_value_t = 10)]
    pub cron_time: u64,

    /// Scheduler tick
    #[clap(short, long, default_value = "1/15 * * * * * *")]
    pub sch_time: String,

    /// Max count per protected endpoint
    #[clap(short, long, default_value_t = 100)]
    pub max_endpoint_count: u64,

    /// Log level
    #[clap(short, long, default_value = "INFO")]
    pub log_level: LogLevel,
}
