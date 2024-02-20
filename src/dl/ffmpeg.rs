use super::spawn::{spawn, SpawnError};

pub struct FFMpeg {}

impl FFMpeg {
    const MP3_BITRATES: [u16; 14] = [
        32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320,
    ];
    pub fn round_mp3_bitrate(abr: f32) -> u16 {
        let abr = abr.ceil() as u16;
        Self::MP3_BITRATES
            .into_iter()
            .find(|f| abr <= *f)
            .unwrap_or(320)
    }

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
