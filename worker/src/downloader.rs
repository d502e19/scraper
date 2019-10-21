
use std::error::Error;
use std::io::{self, Write};
use std::ptr::null;
use std::result::Result;

use reqwest;
use crate::task::Task;
use crate::traits::Downloader;
use std::io::Read;
use futures::Future;


pub(crate) struct DefaultDownloader {}

impl DefaultDownloader {
    pub fn new() -> Self {
        DefaultDownloader {}
    }
}

impl Downloader<Vec<u8>> for DefaultDownloader {
    fn fetch_page(&self, task: Task) -> Result<Vec<u8>, Box<dyn Error>> {

        //convert task.url to str from String
        let mut res = reqwest::get(&*task.url).await?;

        let mut body: Vec<u8> = Vec::new();
        res.read_to_end(&mut body)
            .map(|_| {
                Result::Ok(body);
            })
            .map_err(|err| {
                Result::Err(Box::new((err)))
            })
    }
}