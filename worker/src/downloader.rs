use std::error::Error;
use std::io::{self, Write};
use std::io::Read;
use std::ptr::null;
use std::result::Result;

use futures::Future;
use reqwest;

use crate::task::Task;
use crate::traits::Downloader;

pub(crate) struct DefaultDownloader;


impl Downloader<Vec<u8>> for DefaultDownloader {
    fn fetch_page(&self, task: Task) -> Result<Vec<u8>, Box<dyn Error>> {

        //convert task.url to str from String
        match reqwest::get(&*task.url) {
            Ok(mut res) => {
                let mut body: Vec<u8> = Vec::new();
                match res.read_to_end(&mut body) {
                    Ok(_) => {
                        Ok(body)
                    }
                    Err(e) => {
                        Err(Box::new(e))
                    }
                }
            }
            Err(E) => {
                Err(Box::new(E))
            }
        }
    }
}