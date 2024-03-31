pub mod bot;
pub mod dl;
pub mod notify;
pub mod op;
pub mod request;
pub mod request_chat;
pub mod start;
pub mod types;
pub mod version;

#[macro_export]
macro_rules! reply_i18n_and_return {
    ($bot:expr, $chat_id:expr, $line:expr) => {
        $bot.send_message($chat_id, t!($line)).await?;
        return Ok(())
    };
}

#[macro_export]
macro_rules! parse_integer {
    ($bot:expr, $chat_id:expr, $integer:expr) => {{
        let out: i32 = match $integer.parse() {
            Ok(integer) => integer,
            Err(_) => {
                $bot.send_message($chat_id, t!("not_valid_integer")).await?;
                return Ok(());
            }
        };

        out
    }};
}
