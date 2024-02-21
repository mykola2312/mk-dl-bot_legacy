use std::fs;
use std::path::Path;

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
        Err(_) => false
    }
}

pub fn delete_if_exists(path: &str) {
    if file_exists(path) {
        fs::remove_file(path);
    }
}

pub async fn download(url: &str) -> Result<String, DownloadError> {
    let info = YtDlp::load_info(url).await?;
    let av = match info.best_av_format() {
        Some(av) => av,
        None => return Err(DownloadError::NoFormatFound),
    };
    
    let output_path = make_download_path(&info, &av)?;
    if let Err(e) = YtDlp::download(url, &av.format_id, output_path.as_str()).await {
        delete_if_exists(&output_path);
        return Err(DownloadError::Message(e.to_string()));
    }

    Ok(output_path)
}
