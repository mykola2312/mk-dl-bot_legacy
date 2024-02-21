use dotenv::dotenv;
use std::env;

mod bot;
use bot::bot::bot_main;

mod dl;
use dl::ffmpeg::FFMpeg;
use dl::yt_dlp::YtDlp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let info = YtDlp::load_info(env::var("TEST_URL")?.as_str())
        .await
        .expect("load_info");
    for format in &info.formats {
        println!("{}", format);
    }
    println!("");

    let video = info.best_av_format().unwrap();
    println!("{}", video);

    let audio = info.best_audio_format().unwrap();
    println!("{}", audio);

    println!("abr {}", FFMpeg::round_mp3_bitrate(129.492));

    FFMpeg::convert_to_mp3("audio.m4a", "audio.mp3", 160)
        .await
        .expect("convert");

    Ok(())
    //bot_main().await
}
