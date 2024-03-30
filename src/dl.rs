use std::fmt;
use std::fs;
use tracing::{event, Level};

use crate::dl::ffmpeg::FFMpeg;

use self::spawn::SpawnError;
use self::tmpfile::{TmpFile, TmpFileError};
use self::yt_dlp::{YtDlp, YtDlpError, YtDlpFormat, YtDlpInfo};

pub mod ffmpeg;
mod spawn;
mod tmpfile;
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

impl From<TmpFileError> for DownloadError {
    fn from(value: TmpFileError) -> Self {
        match value {
            TmpFileError::MakePathError => DownloadError::MakePathError,
        }
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

fn make_download_path(
    info: &YtDlpInfo,
    suffix: Option<&str>,
    format: &YtDlpFormat,
) -> Result<String, DownloadError> {
    std::env::temp_dir()
        .join(format!(
            "{}_{}.{}",
            info.id,
            suffix.unwrap_or(""),
            format.ext
        ))
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

async fn download_fallback(url: &str, info: YtDlpInfo) -> Result<TmpFile, DownloadError> {
    let av = match info.best_av_format() {
        Some(av) => av,
        None => {
            event!(
                Level::WARN,
                "no best format found for {}, reverting to default",
                url
            );
            match info.default_format() {
                Some(format) => format,
                None => {
                    event!(Level::ERROR, "no formats found for {}", url);
                    return Err(DownloadError::NoFormatFound);
                }
            }
        }
    };

    Ok(YtDlp::download(url, &info, &av).await?)
}

pub async fn download(url: &str) -> Result<TmpFile, DownloadError> {
    event!(Level::INFO, "url {}", url);

    let info = YtDlp::load_info(url).await?;
    let vf = match info.best_video_format() {
        Some(vf) => vf,
        None => return download_fallback(url, info).await,
    };
    let af = match info.best_audio_format() {
        Some(af) => af,
        None => return download_fallback(url, info).await,
    };

    let video = YtDlp::download(url, &info, &vf).await?;
    let audio = YtDlp::download(url, &info, &af).await?;

    let abr = if let Some(abr) = af.abr {
        FFMpeg::round_mp3_bitrate(abr)
    } else {
        event!(
            Level::ERROR,
            "somehow url {} audio format {} doesnt have abr",
            url,
            af.format_id
        );

        192
    };

    let output = TmpFile::new(format!("{}.{}", &info.id, &vf.ext).as_str())?;
    event!(
        Level::INFO,
        "for {} we joining video {} and audio {}",
        url,
        vf.format_id,
        af.format_id
    );

    let res = FFMpeg::join_video_audio(&video.path, &audio.path, abr, &output.path).await;

    match res {
        Ok(()) => Ok(output),
        Err(e) => Err(DownloadError::Message(e.to_string())),
    }
}
