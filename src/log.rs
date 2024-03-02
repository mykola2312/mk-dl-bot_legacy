use std::io;
use tracing::{level_filters::LevelFilter, Subscriber};
//use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::Filter, layer::SubscriberExt, prelude::*};

//use super::util::VAR_LOG;

// A layer filter to prevent polling timeout errors from clogging logs
// BUG: it suppresses all errors, including from sqlx
struct TeloxideNoiseFilter {}
impl<S: Subscriber> Filter<S> for TeloxideNoiseFilter {
    fn enabled(
        &self,
        meta: &tracing::Metadata<'_>,
        _: &tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        if let Some(module_path) = meta.module_path() {
            if module_path == "teloxide::error_handlers" {
                false
            } else {
                true
            }
        } else {
            true
        }
    }
}

#[cfg(debug_assertions)]
const LOG_LEVEL: LevelFilter = LevelFilter::DEBUG;

#[cfg(not(debug_assertions))]
const LOG_LEVEL: LevelFilter = LevelFilter::INFO;

pub fn log_init() {
    // let log_appender = RollingFileAppender::builder()
    //     .rotation(Rotation::DAILY)
    //     .filename_prefix("mk-dl-bot.log")
    //     .max_log_files(7)
    //     .build(VAR_LOG)
    //     .unwrap();

    // let file_layer = fmt::layer()
    //     .with_ansi(false)
    //     .with_writer(log_appender)
    //     .with_filter(TeloxideNoiseFilter {})
    //     .with_filter(LevelFilter::INFO);

    let stdout_layer = fmt::layer()
        .with_writer(io::stdout)
        //.with_filter(TeloxideNoiseFilter {})
        .with_filter(LOG_LEVEL);

    tracing_subscriber::registry()
        .with(stdout_layer)
        //.with(file_layer)
        .init();
}
