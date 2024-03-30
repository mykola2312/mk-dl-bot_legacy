use super::spawn::{spawn, SpawnError};
use super::tmpfile::{TmpFile, TmpFileError};
use core::fmt;
use ordered_float::OrderedFloat;
use serde::Deserialize;
use serde_json;
use std::fs;
use tracing::{event, Level};

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

struct VideoFormat<'a> {
    pub format: &'a YtDlpFormat,
    pub format_note: &'a String,
    pub width: u16,
    pub height: u16,
    pub vbr: f32,
}

impl<'a> VideoFormat<'a> {
    pub fn is_mp4(&self) -> bool {
        self.format.ext == "mp4"
    }

    pub fn is_premium(&self) -> bool {
        self.format_note.contains("Premium")
    }
}

struct AudioFormat<'a> {
    pub format: &'a YtDlpFormat,
    pub abr: f32,
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
    const H_LIMIT: u16 = 1080;

    pub fn parse(json: &[u8]) -> Result<YtDlpInfo, serde_json::Error> {
        let mut info: YtDlpInfo = serde_json::from_slice(json)?;
        for format in &mut info.formats {
            format.process()
        }

        Ok(info)
    }

    pub fn default_format(&self) -> Option<&YtDlpFormat> {
        match self
            .formats
            .iter()
            .filter(|f| f.height.is_some_and(|h| h <= Self::H_LIMIT))
            .last()
        {
            Some(format) => Some(format),
            None => self.formats.last(),
        }
    }

    #[deprecated(
        since = "0.1.1",
        note = "for YouTube download audio and video separately"
    )]
    pub fn best_av_format(&self) -> Option<&YtDlpFormat> {
        let format = self
            .formats
            .iter()
            .filter_map(|f| {
                if f.vcodec.is_some() && f.acodec.is_some() {
                    Some(VideoFormat {
                        format: &f,
                        format_note: f.format_note.as_ref()?,
                        width: f.width?,
                        height: f.height?,
                        vbr: f.vbr?,
                    })
                } else {
                    None
                }
            })
            .max_by_key(|f| (f.width, f.height));

        match format {
            Some(vf) => Some(vf.format),
            None => {
                event!(Level::ERROR, "no av format for {}", self.id);
                None
            }
        }
    }

    pub fn best_audio_format(&self) -> Option<&YtDlpFormat> {
        let format = self
            .formats
            .iter()
            .filter_map(|f| {
                Some(AudioFormat {
                    format: f,
                    abr: f.abr?,
                })
            })
            .max_by_key(|f| OrderedFloat(f.abr));

        match format {
            Some(af) => Some(af.format),
            None => {
                event!(Level::ERROR, "no audio format for {}", self.id);
                None
            }
        }
    }

    pub fn best_video_format(&self) -> Option<&YtDlpFormat> {
        let format = self
            .formats
            .iter()
            .filter_map(|f| {
                Some(VideoFormat {
                    format: f,
                    format_note: f.format_note.as_ref()?,
                    width: f.width?,
                    height: f.height?,
                    vbr: f.vbr?,
                })
            })
            .filter(|f| f.height <= Self::H_LIMIT && f.is_mp4() && !f.is_premium())
            .max_by_key(|f| OrderedFloat(f.vbr));

        match format {
            Some(vf) => Some(vf.format),
            None => {
                event!(Level::ERROR, "no video format for {}", self.id);
                None
            }
        }
    }
}

#[derive(Debug)]
pub enum YtDlpError {
    SpawnError(SpawnError),
    ErrorMessage(String), // keep it separate type if we ever plan to parse yt-dlp errors
    JsonError,
    NoFormats,
    MakePathError,
    NoFilePresent,
}
// ^(?:ERROR: \[.*\] \S* )(.*$) - regex for matching yt-dlp's youtube errors

impl From<SpawnError> for YtDlpError {
    fn from(value: SpawnError) -> Self {
        match value {
            SpawnError::ErrorMessage(msg) => Self::ErrorMessage(msg),
            _ => Self::SpawnError(value),
        }
    }
}

impl From<TmpFileError> for YtDlpError {
    fn from(_value: TmpFileError) -> Self {
        Self::MakePathError
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
            YTE::SpawnError(e) => write!(f, "{}", e),
            YTE::ErrorMessage(msg) => write!(f, "yt-dlp error - {}", msg),
            YTE::JsonError => write!(f, "json parsing error"),
            YTE::NoFormats => write!(f, "no formats were parsed"),
            YTE::MakePathError => write!(f, "make path error"),
            YTE::NoFilePresent => write!(f, "downloaded file doesn't exists"),
        }
    }
}

pub struct YtDlp {}
impl YtDlp {
    pub async fn load_info(url: &str) -> Result<YtDlpInfo, YtDlpError> {
        let output = spawn("python", &["-m", "yt_dlp", url, "-j"]).await?;

        let info = YtDlpInfo::parse(&output.stdout)?;
        if info.formats.is_empty() {
            return Err(YtDlpError::NoFormats);
        }

        Ok(info)
    }

    pub async fn download(
        url: &str,
        info: &YtDlpInfo,
        format: &YtDlpFormat,
    ) -> Result<TmpFile, YtDlpError> {
        let file =
            TmpFile::new(format!("{}_{}.{}", info.id, format.format_id, format.ext).as_str())?;

        spawn(
            "python",
            &[
                "-m",
                "yt_dlp",
                url,
                "-f",
                &format.format_id,
                "-o",
                &file.path,
                "--force-overwrites",
            ],
        )
        .await?;

        match file.exists() {
            true => Ok(file),
            false => Err(YtDlpError::NoFilePresent),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::YtDlp;
    use std::env;

    #[tokio::test]
    async fn best_av_format() {
        dotenv::from_filename(".env.test").unwrap();
        let info = YtDlp::load_info(env::var("TEST_URL").unwrap().as_str())
            .await
            .unwrap();
        let video = info.best_av_format().unwrap();
        assert_eq!(video.format_id, "22");
    }

    #[tokio::test]
    async fn best_audio_format() {
        dotenv::from_filename(".env.test").unwrap();
        let info = YtDlp::load_info(env::var("TEST_URL").unwrap().as_str())
            .await
            .unwrap();
        let video = info.best_audio_format().unwrap();
        assert_eq!(video.format_id, "140");
    }

    #[tokio::test]
    async fn best_video_format() {
        dotenv::from_filename(".env.test").unwrap();
        let info = YtDlp::load_info(env::var("TEST_URL").unwrap().as_str())
            .await
            .unwrap();
        let video = info.best_video_format().unwrap();
        assert_eq!(video.format_id, "137");
    }
}
