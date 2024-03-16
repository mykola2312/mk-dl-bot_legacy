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
        let bitrate = format!("{}k", bitrate);
        let output = spawn(
            "ffmpeg",
            &[
                "-i",
                input_path,
                "-codec:a",
                "libmp3lame",
                "-b:a",
                bitrate.as_str(),
                "-y",
                output_path,
            ],
        )
        .await?;

        Ok(())
    }

    pub async fn join_video_audio(
        video_path: &str,
        audio_path: &str,
        abr: u16,
        output_path: &str,
    ) -> Result<(), SpawnError> {
        let abr = format!("{}k", abr);
        let output = spawn(
            "ffmpeg",
            &[
                "-i",
                video_path,
                "-i",
                audio_path,
                "-c",
                "copy",
                "-map",
                "0:v:0",
                "-map",
                "1:a:0",
                "-c:a",
                "aac",
                "-b:a",
                &abr,
                output_path,
            ],
        )
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::dl::ffmpeg::FFMpeg;

    #[test]
    fn round_mp3_bitrate() {
        assert_eq!(FFMpeg::round_mp3_bitrate(0.0), 32);
        assert_eq!(FFMpeg::round_mp3_bitrate(30.803), 32);
        assert_eq!(FFMpeg::round_mp3_bitrate(31.957), 32);
        assert_eq!(FFMpeg::round_mp3_bitrate(47.245), 48);
        assert_eq!(FFMpeg::round_mp3_bitrate(62.564), 64);
        assert_eq!(FFMpeg::round_mp3_bitrate(124.216), 128);
        assert_eq!(FFMpeg::round_mp3_bitrate(129.492), 160);
        assert_eq!(FFMpeg::round_mp3_bitrate(320.1), 320);
        assert_eq!(FFMpeg::round_mp3_bitrate(319.4), 320);
        assert_eq!(FFMpeg::round_mp3_bitrate(999.99), 320);
    }
}
