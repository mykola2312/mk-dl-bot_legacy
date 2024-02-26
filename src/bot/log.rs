use super::util::VAR_LOG;
use tracing::Level;
use tracing_appender::{
    non_blocking,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::fmt;

pub fn log_init() {
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("mk-dl-bot.log")
        .max_log_files(7)
        .build(VAR_LOG)
        .unwrap();

    //let (non_blocking, _guard) = non_blocking(file_appender);

    let subscriber = fmt()
        .with_writer(file_appender)
        .with_ansi(true)
        .with_max_level(Level::TRACE)
        .pretty()
        .init();
}
