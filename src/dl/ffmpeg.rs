use super::spawn::{spawn, SpawnError};

pub struct FFMpeg {}

impl FFMpeg {
    pub async fn convert_to_mp3(
        input_path: &str,
        output_path: &str,
        bitrate: u16,
    ) -> Result<(), SpawnError> {
        let output = spawn(
            "ffmpeg",
            [
                "-i",
                input_path,
                "-codec:a",
                "libmp3lame",
                "-b:a",
                "32k",
                output_path,
            ],
        )
        .await?;

        Ok(())
    }
}
