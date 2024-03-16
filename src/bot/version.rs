use teloxide::prelude::*;

use super::types::HandlerResult;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub async fn cmd_version(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, VERSION).await?;

    Ok(())
}
