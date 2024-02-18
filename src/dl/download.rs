use pyo3::prelude::*;
use tokio::task::{spawn_blocking, JoinError};

// TODO: Switch to Command::new for true multithreading and async
pub async fn download_url(url: String) -> Result<bool, JoinError> {
    spawn_blocking(move || {
        let res: PyResult<()> = Python::with_gil(|py| {
            let yt_dlp = PyModule::import(py, "yt_dlp")?;
            let yt = yt_dlp.getattr("YoutubeDL")?;

            let yt_obj = yt.call((), None)?;

            yt_obj.call_method0("__enter__")?;
            yt_obj.call_method1("download", (url,))?;
            yt_obj.call_method0("__exit__")?;

            Ok(())
        });

        match res {
            Ok(_) => true,
            Err(e) => {
                println!("{}", e);
                false
            }
        }
    })
    .await
}
