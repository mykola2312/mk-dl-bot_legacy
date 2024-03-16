use std::fmt;
use std::fs;
use tracing::{event, Level};

use self::spawn::SpawnError;
use self::yt_dlp::{YtDlp, YtDlpError, YtDlpFormat, YtDlpInfo};

pub mod ffmpeg;
mod spawn;
pub mod yt_dlp;

pub enum DownloadError {
    Message(String),
    NoFormatFound,
    MakePathError,
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

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use DownloadError as DE;
        match &self {
            DE::Message(msg) => write!(f, "{}", msg),
            DE::NoFormatFound => write!(
                f,
                "no best format found. you may want to specify one yourself"
            ),
            DE::MakePathError => write!(f, "failed to make path for download file"),
        }
    }
}

fn make_download_path(info: &YtDlpInfo, format: &YtDlpFormat) -> Result<String, DownloadError> {
    std::env::temp_dir()
        .join(format!("{}.{}", info.id, format.ext))
        .into_os_string()
        .into_string()
        .map_err(|e| DownloadError::MakePathError)
}

fn file_exists(path: &str) -> bool {
    match fs::metadata(path) {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn delete_if_exists(path: &str) {
    if file_exists(path) {
        if let Err(e) = fs::remove_file(path) {
            event!(Level::ERROR, "{}", e);
        }
    }
}

pub async fn download(url: &str) -> Result<String, DownloadError> {
    event!(Level::INFO, "url {}", url);

    let info = YtDlp::load_info(url).await?;
    let av = match info.best_av_format() {
        Some(av) => av,
        None => {
            event!(Level::WARN, "no best format found for {}, reverting to default", url);
            match info.default_format() {
                Some(format) => format,
                None => {
                    event!(Level::ERROR, "no formats found for {}", url);
                    return Err(DownloadError::NoFormatFound)
                }
            }
        },
    };

    let output_path = make_download_path(&info, &av)?;
    if let Err(e) = YtDlp::download(url, &av.format_id, output_path.as_str()).await {
        delete_if_exists(&output_path);
        return Err(DownloadError::Message(e.to_string()));
    }

    Ok(output_path)
}
