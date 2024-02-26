use super::util::VAR_LOG;
use tracing::subscriber::set_global_default;
use tracing::Level;
use tracing_appender::{
    non_blocking,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::fmt;

pub fn log_init() {
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("mk-dl-bot.log")
        .max_log_files(7)
        .build(VAR_LOG)
        .unwrap();

    let (non_blocking, guard) = non_blocking(file_appender);
    let file_layer = fmt::layer().with_ansi(true).with_writer(non_blocking);

    let subscriber = fmt()
        .with_ansi(true)
        .with_max_level(Level::TRACE)
        .pretty()
        .finish()
        .with(file_layer);
    set_global_default(subscriber).expect("set_global_default subscriber");
}
