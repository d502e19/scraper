use std::error::Error;
use std::io::Read;

use reqwest;
use reqwest::Client;

use crate::errors::{DownloadError, DownloadErrorKind, DownloadResult};
use crate::task::Task;
use crate::traits::Downloader;
use crate::errors::DownloadErrorKind::{InvalidPage, NetworkError};

/// A struct to access functions in downloader file
/// Contains a reqwest client to send http requests
pub(crate) struct DefaultDownloader {
    client: reqwest::Client
}

impl DefaultDownloader {
    pub fn new() -> Self {
        DefaultDownloader { client: Client::new() }
    }
}

impl Downloader<Vec<u8>> for DefaultDownloader {
    /// Takes a task and returns either a vec<u8> with contents of the url in task, or an error.
    /// If function is successful it will return a Vec<u8> with the page contents, otherwise Error
    fn fetch_page(&self, task: &Task) -> DownloadResult<Vec<u8>> {
        // Attempts to get html from url using the client
        match self.client.get(task.url.as_str()).send() {
            Ok(mut res) => {
                // Read html as bytes into vec
                let mut body: Vec<u8> = Vec::new();
                match res.read_to_end(&mut body) {
                    // If successful return the vec with bytes
                    Ok(_) => Ok(body),
                    // Otherwise error
                    Err(e) => Err(DownloadError::new(InvalidPage, "Could not read downloaded page", Some(Box::new(e)))),
                }
            }
            Err(e) => Err(DownloadError::new(NetworkError, "Failed to download page", Some(Box::new(e)))),
        }
    }
}


#[cfg(test)]
mod tests {
    use mockito::mock;
    use url::Url;

    use crate::downloader::DefaultDownloader;
    use crate::task::Task;
    use crate::traits::Downloader;

    #[test]
    fn test_downloader1() {
        // Url points to a local url to the server
        let url = mockito::server_url();

        // String which is to be the body of the server
        let body = "<!DOCTYPE html>
                            <html>
                            <body>

                            <h1>My first test</h1>
                            <p>This is our first test using mocking (mockito)</p>

                            </body>
                            </html>";

        // Creates expected value for test, which is the body in Vec<u8>
        let expected: Vec<u8> = body.as_bytes().to_vec();

        // Creates the mock
        let _m = mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body(&body)
            .create();

        // Attempts to access the mock and downloads the body
        let dl: DefaultDownloader = DefaultDownloader::new();
        let data = dl.fetch_page(
            &Task { url: Url::parse(&url).unwrap() });

        // Asserts the downloaded data, and expected value is the same
        assert_eq!(data.unwrap(), expected);
    }
}
