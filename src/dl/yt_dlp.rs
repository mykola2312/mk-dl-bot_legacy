use serde::Deserialize;
use serde_json;
use std::process::Output;
use tokio::process::Command;

#[derive(Deserialize, Debug)]
pub struct YtDlpFormat {
    pub format_id: String,
    pub format_note: Option<String>,
    pub audio_channels: Option<u8>,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub ext: String,
    pub vcodec: Option<String>,
    pub acodec: Option<String>,
    pub abr: Option<f32>,
    pub vbr: Option<f32>,
}

#[derive(Deserialize, Debug)]
pub struct YtDlpInfo {
    pub id: String,
    pub title: String,
    pub formats: Vec<YtDlpFormat>,
}

pub enum YtDlpError {
    CommandError(std::io::Error),
}

pub struct YtDlp {}

impl YtDlp {
    pub async fn load_info(url: &str) -> Result<(), YtDlpError> {
        let output = match Command::new("python")
            .args(["-m", "yt_dlp", url, "-j"])
            .output()
            .await
        {
            Ok(output) => output,
            Err(e) => return Err(YtDlpError::CommandError(e)),
        };

        Ok(())
    }
}
