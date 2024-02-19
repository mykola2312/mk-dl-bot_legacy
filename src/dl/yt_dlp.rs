use serde::Deserialize;
use serde_json;
use core::fmt;
use std::str::Utf8Error;
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

#[derive(Debug)]
pub enum YtDlpError {
    CommandError(std::io::Error),
    UtfError(Utf8Error),
    ErrorMessage(String)
}

impl fmt::Display for YtDlpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use YtDlpError as YTE;
        match self {
            YTE::CommandError(e) => write!(f, "Command::new - {}", e),
            YTE::UtfError(_) => write!(f, "Error while decoding UTF8"),
            YTE::ErrorMessage(msg) => write!(f, "yt-dlp error - {}", msg)
        }
    }
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

        if output.stdout.is_empty() && !output.stderr.is_empty() {
            return match std::str::from_utf8(&output.stderr) {
                Ok(message) => Err(YtDlpError::ErrorMessage(message.to_string())),
                Err(utf8_error) => Err(YtDlpError::UtfError(utf8_error))
            };
        }
        Ok(())
    }
}
