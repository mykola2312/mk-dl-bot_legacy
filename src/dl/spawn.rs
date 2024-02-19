use std::process::{Command, Output};
use tokio::task::{spawn_blocking, JoinError};

pub enum SpawnError {
    CommandError(std::io::Error),
    SpawnBlockingError(JoinError),
}

pub async fn spawn_process(program: String, args: Vec<String>) -> Result<Output, SpawnError> {
    let output = spawn_blocking(move || {
        let output = Command::new(program).args(args).output();

        match output {
            Ok(output) => Ok(output),
            Err(e) => Err(SpawnError::CommandError(e)),
        }
    })
    .await;

    match output {
        Ok(output) => output,
        Err(e) => Err(SpawnError::SpawnBlockingError(e)),
    }
}
