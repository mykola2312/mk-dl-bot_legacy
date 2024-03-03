pub mod bot;
pub mod dl;
pub mod notify;
pub mod op;
pub mod request;
pub mod sanitize;
pub mod start;
pub mod types;

#[macro_export]
macro_rules! reply_i18n_and_return {
    ($bot:expr, $chat_id:expr, $line:expr) => {
        $bot.send_message($chat_id, t!($line)).await?;
        return Ok(())
    };
}
