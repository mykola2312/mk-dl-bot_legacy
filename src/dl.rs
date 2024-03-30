use std::fmt;
use tracing::{event, Level};

use crate::bot::sanitize::{extract_url, parse_url};
use crate::dl::ffmpeg::FFMpeg;

use self::spawn::SpawnError;
use self::tmpfile::{TmpFile, TmpFileError};
use self::yt_dlp::{YtDlp, YtDlpError, YtDlpInfo};

pub mod ffmpeg;
mod spawn;
mod tmpfile;
pub mod yt_dlp;

pub enum DownloadError {
    Message(String),
    NotAnURL,
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
            DE::NotAnURL => write!(f, "no url or malformed url were provided"),
            DE::NoFormatFound => write!(
                f,
                "no best format found. you may want to specify one yourself"
            ),
            DE::MakePathError => write!(f, "failed to make path for download file"),
        }
    }
}

enum Downloader {
    Default,
    YouTube,
    TikTok,
}

const DEFAULT_DOWNLOADER: (&'static str, Downloader) = ("", Downloader::Default);
const DOWNLOADERS: [(&'static str, Downloader); 4] = [
    ("www.youtube.com", Downloader::YouTube),
    ("youtu.be", Downloader::YouTube),
    ("www.tiktok.com", Downloader::TikTok),
    ("vm.tiktok.com", Downloader::TikTok),
];

impl Downloader {
    async fn default_download(url: &str, info: &YtDlpInfo) -> Result<TmpFile, DownloadError> {
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

    async fn youtube_download(url: &str, info: &YtDlpInfo) -> Result<TmpFile, DownloadError> {
        let vf = match info.best_video_format() {
            Some(vf) => vf,
            None => return Err(DownloadError::NoFormatFound),
        };
        let af = match info.best_audio_format() {
            Some(af) => af,
            None => return Err(DownloadError::NoFormatFound),
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

    async fn tiktok_download(url: &str, info: &YtDlpInfo) -> Result<TmpFile, DownloadError> {
        let original = info.formats
            .iter()
            .find(|f| f.format_id == "0")
            .ok_or(DownloadError::NoFormatFound)?;

        Ok(YtDlp::download(url, info, original).await?)
    }

    pub async fn download(&self, url: &str, info: &YtDlpInfo) -> Result<TmpFile, DownloadError> {
        match self {
            Downloader::Default => Self::default_download(url, info).await,
            Downloader::YouTube => Self::youtube_download(url, info).await,
            Downloader::TikTok => Self::tiktok_download(url, info).await
        }
    }
}

impl fmt::Display for Downloader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Downloader::Default => write!(f, "Default"),
            Downloader::YouTube => write!(f, "YouTube"),
            Downloader::TikTok => write!(f, "TikTok")
        }
    }
}

pub async fn download(url: &str) -> Result<TmpFile, DownloadError> {
    let url = parse_url(extract_url(url).ok_or(DownloadError::NotAnURL)?)
        .ok_or(DownloadError::NotAnURL)?;
    let host_url = url.host_str().ok_or(DownloadError::NotAnURL)?;
    
    let downloader = &DOWNLOADERS
        .iter()
        .find(|f| f.0 == host_url)
        .unwrap_or(&DEFAULT_DOWNLOADER).1;
    event!(Level::INFO, "using {} downloader for {}", downloader, url);

    let info = YtDlp::load_info(url.as_str()).await?;
    let output = match downloader.download(url.as_str(), &info).await {
        Ok(output) => output,
        Err(e) => {
            event!(Level::ERROR, "downloader {} failed: {}. falling back to default downloader", downloader, e);
            
            DEFAULT_DOWNLOADER.1.download(url.as_str(), &info).await?
        }
    };

    Ok(output)
}
