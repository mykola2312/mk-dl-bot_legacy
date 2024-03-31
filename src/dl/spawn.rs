use core::fmt;
use std::process::Output;
use std::str::Utf8Error;
use std::{fs::OpenOptions, process::Stdio};
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tracing::{event, Level};

use super::tmpfile::TmpFile;

#[derive(Debug)]
pub enum SpawnError {
    CommandError(std::io::Error),
    NoStdErr,
    PipeError(std::io::Error),
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
            FE::NoStdErr => write!(f, "spawned process has closed stderr!"),
            FE::PipeError(e) => write!(f, "pipe error - {}", e),
            FE::UtfError(_) => write!(f, "Error while decoding UTF8"),
            FE::ErrorMessage(msg) => write!(f, "ffmpeg error - {}", msg),
        }
    }
}

/* !!! The argument list could be exploited in a way to inject malicious arguments !!!
!!! and alter the way program executes and/or gain access to system             !!! */
pub async fn spawn(program: &str, args: &[&str]) -> Result<Output, SpawnError> {
    {
        let cmd_args = args.join(" ");
        event!(Level::INFO, "{} {}", program, cmd_args);
    }

    let output = Command::new(program).args(args).output().await?;

    if !output.status.success() {
        let message = std::str::from_utf8(&output.stderr)?;
        return Err(SpawnError::ErrorMessage(message.to_string()));
    }

    Ok(output)
}

pub async fn spawn_pipe(
    program: &str,
    args: &[&str],
    output_file: &TmpFile,
) -> Result<(), SpawnError> {
    {
        let cmd_args = args.join(" ");
        event!(Level::INFO, "{} {}", program, cmd_args);
    }

    let output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&output_file.path)
        .map_err(|e| SpawnError::PipeError(e))?;

    let mut process = Command::new(program)
        .args(args)
        .stdout(output_file)
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stderr = process.stderr.take().ok_or(SpawnError::NoStdErr)?;

    let result = process.wait().await?;

    if !result.success() {
        let mut data: Vec<u8> = Vec::new();
        stderr.read_to_end(&mut data).await?;

        let message = std::str::from_utf8(&data)?;
        return Err(SpawnError::ErrorMessage(message.to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use crate::dl::spawn::{spawn_pipe, SpawnError};
    use crate::dl::tmpfile::TmpFile;

    #[tokio::test]
    async fn test_spawn_pipe() {
        let stdout_file = TmpFile::new("stdout.test").unwrap();
        let result = spawn_pipe(
            "python",
            &[
                "-c",
                "import sys; print('stdout test', end=''); print('stderr test', file=sys.stderr, end=''); sys.exit(1)",
            ],
            &stdout_file,
        )
        .await;

        let stdout = fs::read_to_string(&stdout_file.path).unwrap();
        assert_eq!("stdout test", stdout);

        assert_eq!(true, result.is_err());
        if let Err(e) = result {
            match e {
                SpawnError::ErrorMessage(msg) => assert_eq!("stderr test", msg),
                _ => panic!("SpawnError is not ErrorMessage!")
            }
        }
    }
}