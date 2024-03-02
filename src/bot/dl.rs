use teloxide::prelude::*;
use teloxide::types::InputFile;
use tracing::{event, Level};

use super::types::HandlerResult;
use crate::dl::delete_if_exists;
use crate::dl::download;

async fn bot_download(bot: Bot, msg: Message, url: String) -> HandlerResult {
    let output_path = match download(url.as_str()).await {
        Ok(path) => path,
        Err(e) => {
            event!(Level::ERROR, "{}", e.to_string());
            bot.send_message(msg.chat.id, e.to_string()).await?;
            return Ok(());
        }
    };

    if let Err(e) = bot
        .send_video(msg.chat.id, InputFile::file(&output_path))
        .await
    {
        delete_if_exists(&output_path);
        return Err(Box::new(e));
    }

    Ok(())
}

pub async fn cmd_download(bot: Bot, msg: Message, url: String) -> HandlerResult {
    bot_download(bot, msg, url).await
}
