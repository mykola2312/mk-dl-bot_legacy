use self::spawn::SpawnError;
use self::yt_dlp::YtDlpError;

pub mod ffmpeg;
mod spawn;
pub mod yt_dlp;

pub enum DownloadError {
    Message(String)
}

impl From<SpawnError> for DownloadError {
    fn from(value: SpawnError) -> Self {
        Self::Message(value.to_string())
    }
}

impl From<YtDlpError> for DownloadError {
    fn from(value: YtDlpError) -> Self {
        Self::Message(value.to_string())
    }
}

pub async fn download(url: &str) -> Result<String, DownloadError> {

    todo!()
}