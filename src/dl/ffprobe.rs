use super::spawn::{spawn, SpawnError};
use serde::{de, de::Error, Deserialize, Deserializer};
use std::fmt;

fn duration_from_str<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    
    Ok(str::parse(&s).map_err(de::Error::custom)?)
}

#[derive(Deserialize, Debug)]
pub struct FFProbeStream {
    pub index: u32,
    pub codec_name: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub coded_width: Option<u32>,
    pub coded_height: Option<u32>,
    pub time_base: String,
    pub duration_ts: u64,
    
    #[serde(deserialize_with = "duration_from_str")]
    pub duration: f64
}

impl FFProbeStream {
    pub fn is_video_stream(&self) -> bool {
        self.width.is_some() && self.height.is_some()
    }

    pub fn get_video_resolution(&self) -> Option<(u32, u32)> {
        if self.is_video_stream() {
            return Some((self.width.unwrap(), self.height.unwrap()))
        }

        return None
    }
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

    pub fn get_video_stream(&self) -> Option<&FFProbeStream> {
        self.streams.iter().find(|&s| s.is_video_stream())
    }
}

#[derive(Debug)]
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