use std::error::Error;
use std::io::Read;

use reqwest;

use crate::task::Task;
use crate::traits::Downloader;
use crate::errors::{DownloadResult, DownloadErrorKind, DownloadError};

/// An empty struct to access functions in downloader file
pub(crate) struct DefaultDownloader;


impl Downloader<Vec<u8>> for DefaultDownloader {
    /// If function is successful it will return a Vec<u8> with the page contents, otherwise Error
    fn fetch_page(&self, task: Task) -> DownloadResult<Vec<u8>> {

        // Attempts to get html from url
        match reqwest::get(task.url.as_str()) {
            Ok(mut res) => {
                // Read html as bytes into vec
                let mut body: Vec<u8> = Vec::new();
                match res.read_to_end(&mut body) {
                    // If successful return the vec with bytes
                    Ok(_) => {
                        Ok(body)
                    }
                    // Otherwise error
                    Err(e) => {
                        Err(DownloadError::new(DownloadErrorKind::InvalidPage, String::from("Could not read downloaded page"), Some(Box::new(e))))
                    }
                }
            }
            Err(e) => {
                Err(DownloadError::new(DownloadErrorKind::NetworkError, String::from("Failed to download page"), Some(Box::new(e))))
            }
        }
    }
}