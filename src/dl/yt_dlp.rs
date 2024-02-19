use core::fmt;
use serde::Deserialize;
use serde_json;
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
    pub acodec: Option<String>,
    pub vcodec: Option<String>,
    pub abr: Option<f32>,
    pub vbr: Option<f32>,
}

impl YtDlpFormat {
    pub fn process(&mut self) {
        if self.acodec.as_ref().is_some_and(|v| v == "none") {
            self.acodec = None
        }

        if self.vcodec.as_ref().is_some_and(|v| v == "none") {
            self.vcodec = None
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct YtDlpInfo {
    pub id: String,
    pub title: String,
    pub formats: Vec<YtDlpFormat>,
}

impl YtDlpInfo {
    pub fn process(&mut self) {
        for format in &mut self.formats {
            format.process()
        }
    }
}

#[derive(Debug)]
pub enum YtDlpError {
    CommandError(std::io::Error),
    UtfError(Utf8Error),
    ErrorMessage(String),
    JsonError,
}

impl From<std::io::Error> for YtDlpError {
    fn from(value: std::io::Error) -> Self {
        Self::CommandError(value)
    }
}

impl From<Utf8Error> for YtDlpError {
    fn from(value: Utf8Error) -> Self {
        Self::UtfError(value)
    }
}

impl From<serde_json::Error> for YtDlpError {
    fn from(_value: serde_json::Error) -> Self {
        Self::JsonError
    }
}

impl fmt::Display for YtDlpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use YtDlpError as YTE;
        match self {
            YTE::CommandError(e) => write!(f, "Command::new - {}", e),
            YTE::UtfError(_) => write!(f, "Error while decoding UTF8"),
            YTE::ErrorMessage(msg) => write!(f, "yt-dlp error - {}", msg),
            YTE::JsonError => write!(f, "json parsing error"),
        }
    }
}

pub struct YtDlp {}

impl YtDlp {
    pub async fn load_info(url: &str) -> Result<YtDlpInfo, YtDlpError> {
        let output = Command::new("python")
            .args(["-m", "yt_dlp", url, "-j"])
            .output()
            .await?;

        if output.stdout.is_empty() && !output.stderr.is_empty() {
            let message = std::str::from_utf8(&output.stderr)?;
            return Err(YtDlpError::ErrorMessage(message.to_string()));
        }

        let mut info: YtDlpInfo = serde_json::from_slice(&output.stdout)?;
        info.process();

        Ok(info)
    }
}
