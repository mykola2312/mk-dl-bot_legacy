use serde::Deserialize;
use serde_json;
use std::process::Command;

#[derive(Deserialize, Debug)]
pub struct YtDlpFormat {
    pub format_id: String,
    pub format_note: Option<String>,
    pub audio_channels: Option<u8>,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub ext: String,
    pub vcodec: Option<String>,
    pub acodec: Option<String>,
    pub abr: Option<f32>,
    pub vbr: Option<f32>,
}

#[derive(Deserialize, Debug)]
pub struct YtDlpInfo {
    pub id: String,
    pub title: String,
    pub formats: Vec<YtDlpFormat>,
}

enum YtDlpError {
    SpawnError,
}

pub struct YtDlp {}

impl YtDlp {
    pub async fn load_info(url: &str) {}
}
