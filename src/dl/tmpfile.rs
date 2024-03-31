use std::fs;
use tracing::{event, Level};

#[derive(Debug)]
pub enum TmpFileError {
    MakePathError,
}

pub struct TmpFile {
    pub path: String,
}

impl TmpFile {
    pub fn new(filename: &str) -> Result<Self, TmpFileError> {
        let path = std::env::temp_dir()
            .join(filename)
            .into_os_string()
            .into_string()
            .map_err(|_| TmpFileError::MakePathError)?;

        Ok(Self { path })
    }

    pub fn exists(&self) -> bool {
        match fs::metadata(&self.path) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn delete_if_exists(&self) {
        if self.exists() {
            if let Err(e) = fs::remove_file(&self.path) {
                event!(Level::ERROR, "{}", e);
            }
        }
        event!(Level::INFO, "deleted {}", self.path);
    }
}

impl Drop for TmpFile {
    fn drop(&mut self) {
        self.delete_if_exists();
    }
}
