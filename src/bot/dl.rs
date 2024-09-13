use teloxide::prelude::*;
use teloxide::types::InputFile;
use tracing::{event, Level};

use super::types::HandlerResult;
use crate::dl::download;

use crate::dl::ffprobe::FFProbe;

async fn bot_download(bot: Bot, msg: Message, url: String) -> HandlerResult {
    let output = match download(url.as_str()).await {
        Ok(file) => file,
        Err(e) => {
            event!(Level::ERROR, "{}", e.to_string());
            bot.send_message(msg.chat.id, e.to_string()).await?;
            return Ok(());
        }
    };

    // query media info with
    // ffprobe -v quiet -print_format json -show_streams -select_streams v:0 input.mp4
    let probe = FFProbe::probe(&output.path).await;
    dbg!(probe);

    let mut video = bot.send_video(msg.chat.id, InputFile::file(&output.path));
    // set width, height and so on

    video.await?;

    Ok(())
}

pub async fn cmd_download(bot: Bot, msg: Message, url: String) -> HandlerResult {
    bot_download(bot, msg, url).await
}
