use core::fmt;
use std::{path::Path, str::Utf8Error};
use tokio::process::Command;

pub enum FFMpegError {
    CommandError(std::io::Error),
    UtfError(Utf8Error),
    ErrorMessage(String)
}

impl From<std::io::Error> for FFMpegError {
    fn from(value: std::io::Error) -> Self {
        Self::CommandError(value)
    }
}

impl From<Utf8Error> for FFMpegError {
    fn from(value: Utf8Error) -> Self {
        Self::UtfError(value)
    }
}

impl fmt::Display for FFMpegError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use FFMpegError as FE;
        match self {
            FE::CommandError(e) => write!(f, "Command::new - {}", e),
            FE::UtfError(_) => write!(f, "Error while decoding UTF8"),
            FE::ErrorMessage(msg) => write!(f, "ffmpeg error - {}", msg)
        }
    }
}

pub struct FFMpeg {}

impl FFMpeg {
    pub async fn convert_to_mp3(input_path: &str, output_path: &str, bitrate: u16) -> Result<(), FFMpegError> {
        let output = Command::new("ffmpeg")
            .args(["-i", input_path, "-codec:a", "libmp3lame", "-b:a", "32k", output_path])
            .output()
            .await?;

        if !output.status.success() {
            let message = std::str::from_utf8(&output.stderr)?;
            return Err(FFMpegError::ErrorMessage(message.to_string()));
        }
        
        Ok(())
    }
}