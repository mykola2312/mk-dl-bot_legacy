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

    let mut video = bot.send_video(msg.chat.id, InputFile::file(&output.path));
    // try getting video resolution
    if let Ok(probe) = FFProbe::probe(&output.path).await {
        if let Some(vs) = probe.get_video_stream() {
            if let Some((width, height)) = vs.get_video_resolution() {
                video.width = Some(width);
                video.height = Some(height);
            }

            // set video duration
            video.duration = Some(vs.duration as u32);
        }
    }

    video.await?;

    Ok(())
}

pub async fn cmd_download(bot: Bot, msg: Message, url: String) -> HandlerResult {
    bot_download(bot, msg, url).await
}
