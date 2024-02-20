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
    pub vcodec: Option<String>,
    pub acodec: Option<String>,
    pub vbr: Option<f32>,
    pub abr: Option<f32>,
}

#[derive(Debug)]
struct VideoFormat<'a> {
    pub format: &'a YtDlpFormat,
    pub width: u16,
    pub height: u16,
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

    fn str_option<T>(opt: &Option<T>) -> String
    where
        T: ToString,
    {
        if let Some(value) = opt {
            value.to_string()
        } else {
            "None".to_string()
        }
    }
}

impl fmt::Display for YtDlpFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "id\t{}\text\t{}\tw\t{}\th\t{}\tvcodec\t{}\tvbr\t{}\tacodec\t{}\tabr\t{}",
            self.format_id,
            self.ext,
            Self::str_option(&self.width),
            Self::str_option(&self.height),
            Self::str_option(&self.vcodec),
            Self::str_option(&self.vbr),
            Self::str_option(&self.acodec),
            Self::str_option(&self.abr)
        )
    }
}

#[derive(Deserialize, Debug)]
pub struct YtDlpInfo {
    pub id: String,
    pub title: String,
    pub formats: Vec<YtDlpFormat>,
}

impl YtDlpInfo {
    pub fn parse(json: &[u8]) -> Result<YtDlpInfo, serde_json::Error> {
        let mut info: YtDlpInfo = serde_json::from_slice(json)?;
        for format in &mut info.formats {
            format.process()
        }

        Ok(info)
    }

    pub fn best_video_format(&self) -> Option<&YtDlpFormat> {
        let mut formats: Vec<VideoFormat> = self
            .formats
            .iter()
            .filter_map(|f| {
                if f.vcodec.is_some() && f.acodec.is_some() {
                    Some(VideoFormat {
                        format: &f,
                        width: f.width?,
                        height: f.height?,
                    })
                } else {
                    None
                }
            })
            .collect();
        formats.sort_unstable_by_key(|f| (f.width, f.height));

        match formats.last() {
            Some(vf) => Some(vf.format),
            None => None,
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

        if !output.status.success() {
            let message = std::str::from_utf8(&output.stderr)?;
            return Err(YtDlpError::ErrorMessage(message.to_string()));
        }

        Ok(YtDlpInfo::parse(&output.stdout)?)
    }
}
