use core::fmt;
use std::ffi::OsStr;
use std::process::Output;
use std::str::Utf8Error;
use tokio::process::Command;

#[derive(Debug)]
pub enum SpawnError {
    CommandError(std::io::Error),
    UtfError(Utf8Error),
    ErrorMessage(String),
}

impl From<std::io::Error> for SpawnError {
    fn from(value: std::io::Error) -> Self {
        Self::CommandError(value)
    }
}

impl From<Utf8Error> for SpawnError {
    fn from(value: Utf8Error) -> Self {
        Self::UtfError(value)
    }
}

impl fmt::Display for SpawnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use SpawnError as FE;
        match self {
            FE::CommandError(e) => write!(f, "Command::new - {}", e),
            FE::UtfError(_) => write!(f, "Error while decoding UTF8"),
            FE::ErrorMessage(msg) => write!(f, "ffmpeg error - {}", msg),
        }
    }
}

pub async fn spawn<I, S>(program: &str, args: I) -> Result<Output, SpawnError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new(program).args(args).output().await?;

    if !output.status.success() {
        let message = std::str::from_utf8(&output.stderr)?;
        return Err(SpawnError::ErrorMessage(message.to_string()));
    }

    Ok(output)
}