use std::error::Error;
use std::io::Read;

use reqwest;

use crate::task::Task;
use crate::traits::Downloader;

/// An empty struct to access functions in downloader file
pub(crate) struct DefaultDownloader;


impl Downloader<Vec<u8>> for DefaultDownloader {
    /// If function is successful it will return a Vec<u8> with the page contents, otherwise Error
    fn fetch_page(&self, task: Task) -> Result<Vec<u8>, Box<dyn Error>> {

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
                        Err(Box::new(e))
                    }
                }
            }
            Err(e) => {
                Err(Box::new(e))
            }
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
    use std::net::TcpStream;
    use std::io::{Read, Write};

    #[test]
    fn test_downloader1() {
        let url = mockito::server_url();

        let body = "world";
        let mut mock = mock("GET", "/");
        mock.with_body(&body.to_string()).create();

        let dl: DefaultDownloader = DefaultDownloader;
        let data = dl.fetch_page(Task { url: Url::parse(&url).unwrap() });
        //println!("{:?}", data);
        //println!("{:?}", String::from_utf8(data.unwrap()));

        let mut expected: Vec<u8> = Vec::new();
        expected = body.as_bytes().to_vec();

        assert_eq!(data.unwrap(), expected);
    }
}