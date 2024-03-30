use teloxide::prelude::*;
use teloxide::types::InputFile;
use tracing::{event, Level};

use super::types::HandlerResult;
use crate::dl::download;

async fn bot_download(bot: Bot, msg: Message, url: String) -> HandlerResult {
    let output = match download(url.as_str()).await {
        Ok(file) => file,
        Err(e) => {
            event!(Level::ERROR, "{}", e.to_string());
            bot.send_message(msg.chat.id, e.to_string()).await?;
            return Ok(());
        }
    };

    bot.send_video(msg.chat.id, InputFile::file(&output.path))
        .await?;
    Ok(())
}

pub async fn cmd_download(bot: Bot, msg: Message, url: String) -> HandlerResult {
    bot_download(bot, msg, url).await
}
