use super::spawn::{spawn, SpawnError};
use serde::Deserialize;
use std::fmt;

#[derive(Deserialize, Debug)]
pub struct FFProbeStream {
    pub index: u32,
    pub codec_name: String,
    pub width: u32,
    pub height: u32,
    pub coded_width: u32,
    pub coded_height: u32,
    pub time_base: String,
    pub duration_ts: u64,
    pub duration: f64
}

#[derive(Deserialize, Debug)]
pub struct FFProbeOutput {
    pub streams: Vec<FFProbeStream>
}

impl FFProbeOutput {
    pub fn parse(json: &[u8]) -> Result<FFProbeOutput, serde_json::Error> {
        let output: FFProbeOutput = serde_json::from_slice(json)?;

        Ok(output)
    }
}

pub enum FFProbeError {
    SpawnError(SpawnError),
    JsonError
}

impl From<SpawnError> for FFProbeError {
    fn from(value: SpawnError) -> Self {
        Self::SpawnError(value)
    }
}

impl From<serde_json::Error> for FFProbeError {
    fn from(value: serde_json::Error) -> Self {
        Self::JsonError
    }
}

impl fmt::Display for FFProbeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use FFProbeError as FFPE;
        match (self) {
            FFPE::SpawnError(e) => write!(f, "{}", e),
            FFPE::JsonError => write!(f, "ffprobe json error")
        }
    }
}

pub struct FFProbe {}
impl FFProbe {
    pub async fn probe(input_path: &str) -> Result<FFProbeOutput, FFProbeError> {
        let output = spawn("ffprobe", &[
            "-v", "quiet",
            "-print_format", "json",
            "-show_streams",
            input_path
        ]).await?;

        let output = FFProbeOutput::parse(&output.stdout)?;

        Ok(output)
    }
}